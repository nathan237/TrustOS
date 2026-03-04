#!/usr/bin/env python3
"""
rom_struct14_trace.py — Trace the origin of the [struct+0x14] field
=================================================================
The 5 ATTACKER_CONTROLLED_LOAD findings all load x2 from [x19/x21, #0x14].
This script traces WHERE that structure is populated and whether offset 0x14
comes from attacker-controlled DER data.

Also deep-analyzes the SIZE MISMATCH in 0x10000D5EC:
  heap_alloc(w21) vs memcpy(w20) — are these always safe?
"""

import struct, sys

ROM_PATH = r"C:\Users\nathan\Documents\Scripts\OSrust\tools\checkm8-dualboot\securerom\t8020_B1_securerom.bin"
ROM_BASE = 0x100000000

MEMCPY    = 0x100010BD0
MEMSET    = 0x100010E00
BZERO     = 0x100010D80
HEAP_ALLOC = 0x10000F1EC
PANIC     = 0x100008978

try:
    from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM
    cs = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    cs.detail = True
except ImportError:
    print("ERROR: capstone not available")
    sys.exit(1)

with open(ROM_PATH, "rb") as f:
    rom = f.read()

def rom_read(addr, size):
    off = addr - ROM_BASE
    if 0 <= off < len(rom) - size:
        return rom[off:off+size]
    return None

def disasm_range(start, end):
    data = rom_read(start, end - start)
    if not data:
        return []
    return list(cs.disasm(data, start))

def disasm_func(start, max_size=4096):
    """Disassemble a function until ret."""
    data = rom_read(start, max_size)
    if not data:
        return []
    instrs = []
    for insn in cs.disasm(data, start):
        instrs.append(insn)
        if insn.mnemonic == 'ret' and insn.address > start + 8:
            break
        # Stop at next function prologue (if too far in)
        if insn.address > start + 32 and insn.mnemonic == 'stp' and 'x29' in insn.op_str and 'x30' in insn.op_str:
            break
    return instrs

print("=" * 120)
print("  STRUCT OFFSET 0x14 ORIGIN TRACE + SIZE MISMATCH ANALYSIS")
print("=" * 120)

# =====================================================================================
# PART 1: What is at [struct+0x14]?
# Find ALL references to offset 0x14 in the DER/cert region
# =====================================================================================
print(f"\n{'#' * 120}")
print("# PART 1: ALL REFERENCES TO STRUCT OFFSET 0x14 IN DER/CERT REGIONS")
print(f"{'#' * 120}")

# Scan all code in the cert/DER regions for any instruction touching offset 0x14
regions = [
    (0x10000D000, 0x10000F000, "DER/ASN.1 Core"),
    (0x100012000, 0x100015200, "Cert/X.509/DER Deep"),
]

offset_14_refs = []
for rstart, rend, rname in regions:
    instrs = disasm_range(rstart, rend)
    for insn in instrs:
        ops = insn.op_str
        # Look for any instruction that reads/writes offset 0x14
        if '#0x14' in ops or ', #0x14]' in ops:
            offset_14_refs.append((insn.address, insn.mnemonic, ops, rname))

print(f"\n  Found {len(offset_14_refs)} references to offset 0x14:\n")
for addr, mn, ops, rname in offset_14_refs:
    # Classify: read (ldr) or write (str)
    if mn.startswith('ldr'):
        kind = "READ"
    elif mn.startswith('str'):
        kind = "WRITE"
    else:
        kind = "OTHER"
    print(f"  [{kind:5s}] 0x{addr:X}: {mn:8s} {ops:50s} ({rname})")

# =====================================================================================
# PART 2: Where are the WRITES to offset 0x14?
# If the struct[0x14] is written from DER data → attacker controlled
# =====================================================================================
print(f"\n\n{'#' * 120}")
print("# PART 2: TRACING WRITES TO OFFSET 0x14 — WHO POPULATES THIS FIELD?")
print(f"{'#' * 120}")

# Find all STR instructions that write to [reg, #0x14]
writes = [(a, m, o, r) for a, m, o, r in offset_14_refs if m.startswith('str')]
print(f"\n  {len(writes)} write sites to offset 0x14:\n")

for addr, mn, ops, rname in writes:
    print(f"\n  --- WRITE at 0x{addr:X}: {mn} {ops} ---")
    
    # Get surrounding context (20 instrs before)
    context = disasm_range(addr - 80, addr + 8)
    
    # Find the source register
    src_reg = ops.split(',')[0].strip()
    print(f"    Source register: {src_reg}")
    
    # Trace backward to find where src_reg was set
    found = False
    for insn in reversed(context):
        if insn.address >= addr:
            continue
        parts = insn.op_str.split(',')
        dest = parts[0].strip() if parts else ""
        
        # Check if this sets the source register (or its width variant)
        reg_variants = {src_reg}
        if src_reg.startswith('w'):
            reg_variants.add('x' + src_reg[1:])
        elif src_reg.startswith('x'):
            reg_variants.add('w' + src_reg[1:])
        
        if dest in reg_variants:
            print(f"    Origin: 0x{insn.address:X}: {insn.mnemonic} {insn.op_str}")
            
            # Is it from a constant?
            rest = ','.join(parts[1:]).strip()
            if rest.startswith('#'):
                print(f"    → CONSTANT VALUE: {rest}")
            elif insn.mnemonic.startswith('ldr'):
                print(f"    → LOADED FROM MEMORY: {rest}")
                # This might be from DER data!
            elif insn.mnemonic == 'mov':
                print(f"    → REGISTER: {rest} — need deeper trace")
            elif insn.mnemonic in ('add', 'sub', 'and', 'orr', 'lsl', 'lsr'):
                print(f"    → COMPUTED: {insn.op_str}")
            found = True
            break
    
    if not found:
        print(f"    Origin: COULD NOT TRACE")
    
    # Show context
    print(f"    Context (10 instructions before write):")
    for insn in context:
        if insn.address <= addr:
            marker = " <<<" if insn.address == addr else ""
            print(f"      0x{insn.address:X}: {insn.mnemonic:8s} {insn.op_str}{marker}")

# =====================================================================================
# PART 3: Deep analysis of 0x10000E9F0 — DER chain builder
# Focus on the data flow: where does x21/x19 point to?
# =====================================================================================
print(f"\n\n{'#' * 120}")
print("# PART 3: FULL ANNOTATED DISASM OF 0x10000E9F0 (DER chain builder)")
print("# Focus: Where does x21 point? What is at [x21, #0x14]?")
print(f"{'#' * 120}")

instrs = disasm_range(0x10000E9F0, 0x10000E9F0 + 800)
print(f"\n  Function: 0x10000E9F0 ({len(instrs)} instructions)\n")

# Track register assignments
reg_info = {}
for insn in instrs:
    ops = insn.op_str
    mn = insn.mnemonic
    parts = ops.split(',')
    dest = parts[0].strip() if parts else ""
    
    annotation = ""
    
    # Track x21 assignment (the struct pointer)
    if dest in ('x21', 'w21'):
        if mn == 'mov':
            src = parts[1].strip() if len(parts) > 1 else "?"
            annotation = f"  // x21 = {src}"
        elif mn.startswith('ldr'):
            annotation = f"  // x21 = MEM[{','.join(parts[1:]).strip()}]"
    
    # Highlight loads from offset 0x14
    if '#0x14' in ops:
        if mn.startswith('ldr'):
            annotation = "  // *** LOAD FROM STRUCT+0x14 ***"
        elif mn.startswith('str'):
            annotation = "  // *** STORE TO STRUCT+0x14 ***"
    
    # Highlight memcpy calls
    if mn == 'bl':
        try:
            target = int(ops.replace('#',''), 16)
            if target == MEMCPY:
                annotation = "  // >>> MEMCPY(x0=dst, x1=src, x2=SIZE) <<<"
            elif target == BZERO:
                annotation = "  // bzero"
            elif target == HEAP_ALLOC:
                annotation = "  // heap_alloc"
        except:
            pass
    
    # Highlight x2 setup
    if dest in ('x2', 'w2'):
        annotation += "  // ** SETS SIZE x2 **"
    
    # Highlight bounds checks
    if mn in ('cmp', 'cmn', 'subs', 'tbnz', 'tbz', 'cbz', 'cbnz'):
        annotation += "  // ** CHECK **"
    
    raw = rom_read(insn.address, insn.size)
    hexbytes = raw.hex() if raw else "????????"
    print(f"  0x{insn.address:X}: {hexbytes:8s}  {mn:8s} {ops:45s}{annotation}")
    
    if mn == 'ret' and insn.address > 0x10000E9F0 + 16:
        break

# =====================================================================================
# PART 4: Deep analysis of 0x10000ECD0 — DER chain ops
# The most dangerous: lsl x2, x28, x8 where x8 = ldr w8, [x19, #0x14]
# =====================================================================================
print(f"\n\n{'#' * 120}")
print("# PART 4: FULL ANNOTATED DISASM OF 0x10000ECD0 (DER chain ops)")  
print("# DANGER: lsl x2, x28, x8 where x8 = ldr [x19, #0x14]")
print(f"{'#' * 120}")

instrs = disasm_range(0x10000ECD0, 0x10000ECD0 + 700)
print(f"\n  Function: 0x10000ECD0 ({len(instrs)} instructions)\n")

for insn in instrs:
    ops = insn.op_str
    mn = insn.mnemonic
    parts = ops.split(',')
    dest = parts[0].strip() if parts else ""
    
    annotation = ""
    
    if '#0x14' in ops:
        if mn.startswith('ldr'):
            annotation = "  // *** LOAD FROM STRUCT+0x14 — ATTACKER? ***"
        elif mn.startswith('str'):
            annotation = "  // *** STORE TO STRUCT+0x14 ***"
    
    if '#0x30' in ops and mn.startswith('ldr'):
        annotation += "  // load function pointer?"
    
    if mn == 'lsl' and 'x2' in ops:
        annotation = "  // !!! LEFT SHIFT ON SIZE — DANGEROUS !!!"
    
    if mn == 'bl':
        try:
            target = int(ops.replace('#',''), 16)
            if target == MEMCPY:
                annotation = "  // >>> MEMCPY(x0=dst, x1=src, x2=SIZE) <<<"
            elif target == BZERO:
                annotation = "  // bzero"
            elif target == HEAP_ALLOC:
                annotation = "  // heap_alloc"
            elif target == 0x10000EF1C:
                annotation = "  // sub-function call"
            elif target == 0x10000E82C:
                annotation = "  // sub-function call"
            elif target == 0x10000F468:
                annotation = "  // sub-function call (cleanup?)"
        except:
            pass
    
    if mn == 'blr':
        annotation = "  // INDIRECT CALL via register"
    
    if dest in ('x2', 'w2'):
        annotation += "  // ** SETS SIZE x2 **"
    
    if mn in ('cmp', 'cmn', 'subs', 'tbnz', 'tbz', 'cbz', 'cbnz'):
        annotation += "  // ** CHECK **"
    
    if mn == 'asr':
        annotation += "  // arithmetic shift right"
    
    raw = rom_read(insn.address, insn.size)
    hexbytes = raw.hex() if raw else "????????"
    print(f"  0x{insn.address:X}: {hexbytes:8s}  {mn:8s} {ops:45s}{annotation}")
    
    if mn == 'ret' and insn.address > 0x10000ECD0 + 16:
        break

# =====================================================================================
# PART 5: Size mismatch analysis in 0x10000D5EC
# =====================================================================================
print(f"\n\n{'#' * 120}")
print("# PART 5: SIZE MISMATCH IN 0x10000D5EC — heap_alloc(w21) vs memcpy(w20)")
print(f"{'#' * 120}")

# Let's trace the exact computation of w21 and w20
instrs = disasm_range(0x10000D5EC, 0x10000D5EC + 900)

# Find the two key reassignment points
print(f"\n  Tracing w21 (allocation size) and w20 (memcpy size):\n")
print(f"  ALLOCATION SIZE LOOP (accumulates w21):")

in_alloc_loop = False
for insn in instrs:
    mn = insn.mnemonic
    ops = insn.op_str
    
    # Show the relevant parts
    if insn.address >= 0x10000D654 and insn.address <= 0x10000D6F8:
        parts = ops.split(',')
        dest = parts[0].strip() if parts else ""
        
        annotation = ""
        if dest in ('w21', 'x21') or 'w21' in ops:
            annotation = "  // *** w21 (alloc size) ***"
        if dest in ('w20', 'x20') or 'w20' in ops:
            annotation = "  // *** w20 (memcpy size) ***"
        if mn == 'bl':
            try:
                target = int(ops.replace('#',''), 16)
                if target == HEAP_ALLOC:
                    annotation = "  // !!! HEAP_ALLOC(x0=x19=sxtw(w21)) !!!"
            except:
                pass
        if 'x19' in ops and mn in ('sxtw', 'mov'):
            annotation += "  // x19 assignment"
        if mn == 'sxtw':
            annotation += f"  // sign-extend {ops}"
        
        raw = rom_read(insn.address, insn.size)
        hexbytes = raw.hex() if raw else "????????"
        print(f"    0x{insn.address:X}: {hexbytes:8s}  {mn:8s} {ops:40s}{annotation}")

print(f"\n  INNER LOOP WITH MEMCPY (uses x19):")
for insn in instrs:
    mn = insn.mnemonic
    ops = insn.op_str
    
    if insn.address >= 0x10000D728 and insn.address <= 0x10000D7FC:
        parts = ops.split(',')
        dest = parts[0].strip() if parts else ""
        
        annotation = ""
        if 'x19' in ops:
            annotation = "  // *** uses x19 ***"
        if mn == 'sxtw' and 'w20' in ops:
            annotation = "  // !!! x19 = sxtw(w20) — DIFFERENT from alloc size! !!!"
        if mn == 'bl':
            try:
                target = int(ops.replace('#',''), 16)
                if target == MEMCPY:
                    annotation = "  // >>> MEMCPY — x2=x19=sxtw(w20) <<<"
            except:
                pass
        if dest in ('x2', 'w2'):
            annotation += "  // SETS SIZE"
        
        raw = rom_read(insn.address, insn.size)
        hexbytes = raw.hex() if raw else "????????"
        print(f"    0x{insn.address:X}: {hexbytes:8s}  {mn:8s} {ops:40s}{annotation}")

# Arithmetic analysis
print(f"""
  SIZE COMPUTATION ANALYSIS:
  
  w21 (alloc size) = 9 + Σ over entries:
    entry_count = ldrsw [x22, #0x10]  (from global — stored during DER parse)
    For each entry i:
      w12 = csel(ldr [entry+0], 0, gt) * 9 + w12  (w12 starts as 9)
      w13 = ldr [entry+0x10]  (field count)
      w14 = Σ ldrb [entry+0x18 + j] for j in 0..w13  (accumulated byte values)
      w20 += w14   (raw data size sum)
      w10 = csel(ldr [entry+0x20], 0, gt) * 7  (overhead per sub-field)
      w21 = w12 + w10
  
  w20 (memcpy size) = Σ of data byte values only (no overhead)
  
  Therefore: w21 = w20 + overhead_per_entry + 9_per_entry
  
  CONCLUSION: w21 >= w20 ALWAYS (because w21 includes w20's contribution plus
  additional per-entry overhead of at least 9 bytes).
  
  This means the allocation is ALWAYS large enough for the memcpy.
  → SIZE MISMATCH IS NOT EXPLOITABLE (w21 > w20 guaranteed)
""")

# =====================================================================================
# PART 6: Trace callers of 0x10000E9F0 and 0x10000ECD0
# Who creates the struct that has offset 0x14?
# =====================================================================================
print(f"\n{'#' * 120}")
print("# PART 6: WHO CALLS 0x10000E9F0 AND 0x10000ECD0?")
print("# Tracing how the struct (with offset 0x14) is created")
print(f"{'#' * 120}")

# Search the entire codebase for BL to these functions
targets_to_find = {
    0x10000E9F0: "DER chain builder",
    0x10000ECD0: "DER chain ops",
}

for target, name in targets_to_find.items():
    print(f"\n  Searching for callers of 0x{target:X} ({name})...")
    
    # Scan all code
    callers = []
    all_code = disasm_range(0x100000000, 0x10001C000)
    for insn in all_code:
        if insn.mnemonic == 'bl':
            try:
                bl_target = int(insn.op_str.replace('#',''), 16)
                if bl_target == target:
                    callers.append(insn.address)
            except:
                pass
    
    print(f"  Found {len(callers)} call sites:")
    for caller in callers:
        # Show context around the call (what was passed as x0/x1 = the struct)
        print(f"\n    Call at 0x{caller:X}:")
        context = disasm_range(caller - 48, caller + 8)
        for insn in context:
            marker = " <<<< CALL" if insn.address == caller else ""
            # What sets x0 (first arg = the struct)?
            parts = insn.op_str.split(',')
            dest = parts[0].strip() if parts else ""
            if dest in ('x0', 'w0'):
                marker += " [SETS x0=struct]"
            elif dest in ('x1', 'w1'):
                marker += " [SETS x1]"
            elif dest in ('x2', 'w2'):
                marker += " [SETS x2]"
            elif dest in ('x3', 'w3'):
                marker += " [SETS x3]"
            print(f"      0x{insn.address:X}: {insn.mnemonic:8s} {insn.op_str}{marker}")

# =====================================================================================
# PART 7: What is the struct at [x0+0x14]? Trace global data
# =====================================================================================
print(f"\n\n{'#' * 120}")
print("# PART 7: WHAT IS THE STRUCT TYPE? ANALYZING OFFSET 0x14")
print(f"{'#' * 120}")

# The functions use x19/x21 which is typically the first arg (x0) or loaded from global
# Let's check what's stored at the common global addresses

# From 0x10000E9F0 prologue: x21 = x0 (first argument)
# So [x0+0x14] is the 5th word (offset 20) of the argument structure

# Let me check ALL accesses in 0x10000E9F0 to understand the struct layout
print(f"\n  Struct layout used by 0x10000E9F0:")
print(f"  (x21 = x0 = first argument = struct pointer)\n")

instrs = disasm_range(0x10000E9F0, 0x10000E9F0 + 800)
struct_accesses = {}
for insn in instrs:
    ops = insn.op_str
    mn = insn.mnemonic
    
    # Find all accesses to [x21, #offset] or [x19, #offset]
    for base in ['x21', 'x19']:
        if f'[{base}' in ops:
            # Extract the offset
            import re
            m = re.search(rf'\[{base}(?:, #(0x[0-9a-f]+|[0-9]+))?\]', ops)
            if m:
                off_str = m.group(1) if m.group(1) else "0"
                offset = int(off_str, 16) if off_str.startswith('0x') else int(off_str)
                key = (base, offset)
                if key not in struct_accesses:
                    struct_accesses[key] = []
                rw = "R" if mn.startswith('ldr') else "W" if mn.startswith('str') else "?"
                struct_accesses[key].append((insn.address, rw, mn, ops))

# Print sorted by offset
for (base, offset), accesses in sorted(struct_accesses.items(), key=lambda x: x[0][1]):
    print(f"\n  [{base}+0x{offset:X}] ({len(accesses)} accesses):")
    for addr, rw, mn, ops in accesses:
        print(f"    [{rw}] 0x{addr:X}: {mn} {ops}")

# Also do the same for 0x10000ECD0
print(f"\n\n  Struct layout used by 0x10000ECD0:")
print(f"  (x19 = x0 = first argument = struct pointer)\n")

instrs = disasm_range(0x10000ECD0, 0x10000ECD0 + 700)
struct_accesses = {}
for insn in instrs:
    ops = insn.op_str
    mn = insn.mnemonic
    
    for base in ['x19', 'x21']:
        if f'[{base}' in ops:
            import re
            m = re.search(rf'\[{base}(?:, #(0x[0-9a-f]+|[0-9]+))?\]', ops)
            if m:
                off_str = m.group(1) if m.group(1) else "0"
                offset = int(off_str, 16) if off_str.startswith('0x') else int(off_str)
                key = (base, offset)
                if key not in struct_accesses:
                    struct_accesses[key] = []
                rw = "R" if mn.startswith('ldr') else "W" if mn.startswith('str') else "?"
                struct_accesses[key].append((insn.address, rw, mn, ops))

for (base, offset), accesses in sorted(struct_accesses.items(), key=lambda x: x[0][1]):
    print(f"\n  [{base}+0x{offset:X}] ({len(accesses)} accesses):")
    for addr, rw, mn, ops in accesses:
        print(f"    [{rw}] 0x{addr:X}: {mn} {ops}")

# =====================================================================================
# PART 8: Trace the initialization of the struct
# Where is [struct+0x14] first written?
# =====================================================================================
print(f"\n\n{'#' * 120}")
print("# PART 8: GLOBAL SEARCH — WHO WRITES TO [any_reg, #0x14] IN ALL DER/CERT CODE?")
print(f"{'#' * 120}")

# This will find the initialization point
all_writes_14 = []
all_code = disasm_range(0x10000D000, 0x100015200)
for insn in all_code:
    mn = insn.mnemonic
    ops = insn.op_str
    if mn.startswith('str') and '#0x14]' in ops:
        all_writes_14.append((insn.address, mn, ops))

print(f"\n  Found {len(all_writes_14)} STR instructions writing to offset 0x14:\n")
for addr, mn, ops in all_writes_14:
    # Get the source register and trace it
    src = ops.split(',')[0].strip()
    
    # Get 5 instructions before for context
    context = disasm_range(addr - 20, addr + 4)
    
    # Find what sets the source register
    origin = "?"
    for insn in reversed(context):
        if insn.address >= addr:
            continue
        parts = insn.op_str.split(',')
        dest = parts[0].strip() if parts else ""
        if dest == src or (src.startswith('w') and dest == 'x' + src[1:]) or (src.startswith('x') and dest == 'w' + src[1:]):
            rest = ','.join(parts[1:]).strip()
            origin = f"0x{insn.address:X}: {insn.mnemonic} {insn.op_str}"
            break
    
    print(f"  0x{addr:X}: {mn:8s} {ops:50s} src={src}, origin={origin}")

# =====================================================================================
# FINAL ASSESSMENT
# =====================================================================================
print(f"\n\n{'#' * 120}")
print("# FINAL ASSESSMENT: IS [struct+0x14] ATTACKER-CONTROLLED?")
print(f"{'#' * 120}")

print("""
  The field at offset 0x14 in the DER/cert structure represents a TYPE DESCRIPTOR
  property — likely the "bit width" or "element size" of an ASN.1 type.
  
  ANALYSIS CHAIN:
  1. DER data comes in from USB (DFU mode) → 0x10000D0A8 copies to global
  2. DER parser (0x10000D5EC etc.) parses TLV structures
  3. Creates internal type descriptors with fields:
     - +0x00: raw data pointer
     - +0x08: context/flags
     - +0x10: length
     - +0x14: element_size / bit_width  ← THE CRITICAL FIELD
     - +0x18: sub-elements pointer
     - +0x30: vtable / function pointer
  
  4. These descriptors are passed to 0x10000E9F0/0x10000ECD0 which use
     [struct+0x14] to compute memcpy sizes
  
  KEY QUESTION: Does [struct+0x14] come directly from the DER stream,
  or is it derived from hardcoded type tables?
  
  If from DER stream → EXPLOITABLE (attacker controls the shift amount in lsl)
  If from type tables → Safe (ROM-resident constants)
  
  The lsl x2, x28, x8 pattern at 0x10000EE10 is especially dangerous:
    x8 = ldr w8, [x19, #0x14]  ← if this is e.g. 32, then x2 = x28 << 32 = 0
    But if x8 = 5, x2 = x28 << 5 = x28 * 32 → could overflow buffer
""")

print("=" * 120)
print("  ANALYSIS COMPLETE")
print("=" * 120)
