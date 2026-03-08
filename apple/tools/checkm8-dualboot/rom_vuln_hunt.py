#!/usr/bin/env python3
"""
ROM Vulnerability Hunter - Targeted analysis based on A0→B1 diff findings
 
Key insights from diffing:
1. A0 had 32-bit comparison where B1 uses 64-bit (integer truncation bug fix)
2. A0 didn't zero stack buffers that B1 does (info leak fix)
3. A0 used hardcoded 0x1B0000 sizes that B1 validates dynamically
4. New functions in B1 added address range validation

Now we hunt for:
- Similar 32-bit/64-bit comparison patterns remaining in B1
- The DFU handler and USB request processing (proper identification)
- Heap management patterns (checkm8 was use-after-free)
- Integer overflow/truncation in size calculations
- Missing bounds checks on user-controlled data
"""

import struct
from capstone import *

ROM_BASE = 0x100000000

def load_rom(path):
    with open(path, 'rb') as f:
        return f.read()

def disassemble(data, offset, count=None, size=None):
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    if size:
        code = data[offset:offset+size]
    elif count:
        code = data[offset:offset+count*4+100]
    else:
        code = data[offset:offset+0x1000]
    return list(md.disasm(code, ROM_BASE + offset))

def disasm_function(data, offset, max_insns=500):
    """Disassemble a function until RET"""
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    code = data[offset:offset+max_insns*4]
    insns = []
    for insn in md.disasm(code, ROM_BASE + offset):
        insns.append(insn)
        if insn.mnemonic == 'ret':
            break
    return insns

def find_all_functions(data, code_end=0x24B40):
    """Find all function prologues"""
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    funcs = []
    code = data[:code_end]
    insns = list(md.disasm(code, ROM_BASE))
    
    for i, insn in enumerate(insns):
        if insn.mnemonic == 'stp' and 'x29' in insn.op_str and 'x30' in insn.op_str:
            funcs.append(insn.address)
    return funcs

# =====================================================================================================================
# HUNT 1: 32-bit vs 64-bit comparison patterns in B1
# Looking for places where a 32-bit compare (cmp wN, wM) might be insufficient
# Especially after loading with ldr wN (which zero-extends) but before using as 64-bit
# =====================================================================================================================
def hunt_integer_truncation(data, rom_name="B1"):
    print(f"\n{'='*120}")
    print(f"  HUNT 1: INTEGER TRUNCATION PATTERNS IN {rom_name}")
    print(f"  Looking for 32-bit comparisons on values that might need 64-bit checks")
    print(f"{'='*120}\n")
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    code = data[:0x24B40]
    insns = list(md.disasm(code, ROM_BASE))
    
    findings = []
    
    for i in range(len(insns) - 5):
        # Pattern 1: ldr wN, [addr] then cmp wN, wM (instead of cmp xN, xM)
        # This was the exact bug Apple fixed
        if insns[i].mnemonic == 'ldr' and insns[i].op_str.startswith('w'):
            load_reg = insns[i].op_str.split(',')[0].strip()  # e.g. 'w2'
            load_addr = insns[i].address
            
            # Look ahead for comparison using this register (within 10 instructions)
            for j in range(i+1, min(i+15, len(insns))):
                if insns[j].mnemonic == 'cmp':
                    ops = insns[j].op_str.split(',')
                    cmp_reg = ops[0].strip()
                    
                    if cmp_reg == load_reg:
                        # Found: 32-bit load followed by 32-bit comparison
                        # Check if the comparison target is also 32-bit
                        cmp_target = ops[1].strip()
                        if cmp_target.startswith('w'):
                            # Now check if the loaded value is later used as 64-bit
                            x_reg = 'x' + load_reg[1:]
                            for k in range(j+1, min(j+20, len(insns))):
                                if x_reg in insns[k].op_str:
                                    findings.append({
                                        'type': 'W_LOAD_W_CMP_X_USE',
                                        'load_addr': load_addr,
                                        'cmp_addr': insns[j].address,
                                        'use_addr': insns[k].address,
                                        'load': f"{insns[i].mnemonic} {insns[i].op_str}",
                                        'cmp': f"{insns[j].mnemonic} {insns[j].op_str}",
                                        'use': f"{insns[k].mnemonic} {insns[k].op_str}",
                                        'reg': load_reg,
                                    })
                                    break
                        break
        
        # Pattern 2: sxtw (sign-extend 32→64) used before comparison
        # If sxtw is missing, negative 32-bit values would be zero-extended
        if insns[i].mnemonic == 'ldr' and insns[i].op_str.startswith('w'):
            load_reg = insns[i].op_str.split(',')[0].strip()
            x_reg = 'x' + load_reg[1:]
            
            # Check if this register is used as x-register without sxtw
            has_sxtw = False
            used_as_x = False
            use_insn = None
            
            for j in range(i+1, min(i+20, len(insns))):
                if insns[j].mnemonic == 'sxtw' and x_reg in insns[j].op_str:
                    has_sxtw = True
                    break
                if insns[j].mnemonic in ('bl', 'ret', 'b'):
                    break
                # Check if x-variant is used as operand
                ops = insns[j].op_str.split(',')
                for op in ops[1:]:  # skip destination
                    if x_reg in op.strip() and insn.mnemonic not in ('stp', 'ldp'):
                        used_as_x = True
                        use_insn = insns[j]
                        break
            
            if used_as_x and not has_sxtw and use_insn:
                # Potential sign-extension issue
                pass  # Too noisy, skip

    # Pattern 3: Size/length stored as 32-bit but buffer operations use 64-bit
    print(f"  Found {len(findings)} potential integer truncation patterns:\n")
    
    # Deduplicate by cmp_addr
    seen = set()
    unique = []
    for f in findings:
        if f['cmp_addr'] not in seen:
            seen.add(f['cmp_addr'])
            unique.append(f)
    
    for f in unique:
        print(f"    [{f['type']}] @ 0x{f['cmp_addr']:X}")
        print(f"      Load:    0x{f['load_addr']:X}: {f['load']}")
        print(f"      Compare: 0x{f['cmp_addr']:X}: {f['cmp']}")
        print(f"      64-use:  0x{f['use_addr']:X}: {f['use']}")
        print()
    
    return unique


# =====================================================================================================================
# HUNT 2: DFU HANDLER IDENTIFICATION (proper method)
# The DFU handler processes USB control transfers. We need to find it by:
# 1. Finding USB-related strings
# 2. Finding MMIO patterns for USB controller
# 3. Following the USB setup packet handling code
# =====================================================================================================================
def hunt_dfu_handler(data, rom_name="B1"):
    print(f"\n{'='*120}")
    print(f"  HUNT 2: DFU HANDLER & USB REQUEST PROCESSING IN {rom_name}")
    print(f"{'='*120}\n")
    
    # Step 1: Find ALL interesting strings
    strings_found = {}
    for i in range(len(data) - 4):
        # Look for printable ASCII strings >= 4 chars
        if data[i:i+4] == b'USB ' or data[i:i+3] == b'DFU' or data[i:i+4] == b'DONE' or \
           data[i:i+4] == b'DNLD' or data[i:i+4] == b'usb-' or data[i:i+4] == b'DnDn':
            end = i
            while end < len(data) and data[end] >= 0x20 and data[end] < 0x7f:
                end += 1
            if end - i >= 3:
                s = data[i:end].decode('ascii', errors='replace')
                strings_found[i] = s
    
    # Find more USB/DFU related strings
    search_terms = [b'Apple Mobile Device (DFU Mode)', b'DFU', b'USBCFG', b'usb_core', 
                    b'SETUP', b'EP0', b'CTRL', b'transfer', b'request',
                    b'bRequest', b'bmRequestType', b'wValue', b'wLength',
                    b'apple-fail', b'CPID:', b'SRTG:', b'SDOM:', b'NONCE:',
                    b'Apple Inc', b'iBoot']
    
    for term in search_terms:
        pos = 0
        while True:
            idx = data.find(term, pos)
            if idx == -1:
                break
            # Extend to full string
            start = idx
            while start > 0 and data[start-1] >= 0x20 and data[start-1] < 0x7f:
                start -= 1
            end = idx + len(term)
            while end < len(data) and data[end] >= 0x20 and data[end] < 0x7f:
                end += 1
            s = data[start:end].decode('ascii', errors='replace')
            strings_found[start] = s
            pos = idx + len(term)
    
    print(f"  USB/DFU related strings found:")
    for addr, s in sorted(strings_found.items()):
        print(f"    0x{addr:05X} (VA 0x{ROM_BASE + addr:X}): \"{s}\"")
    
    # Step 2: Find ALL ADRP instructions and their target pages
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    code = data[:0x24B40]
    insns = list(md.disasm(code, ROM_BASE))
    
    # Build ADRP+ADD pairs to find code that references these strings
    string_refs = {}  # string_addr -> [(code_addr, adrp_insn, add_insn)]
    
    for i in range(len(insns) - 1):
        if insns[i].mnemonic == 'adrp':
            # Extract page address
            ops = insns[i].op_str.split(',')
            if len(ops) >= 2:
                try:
                    page_val = int(ops[1].strip().replace('#', ''), 16) & ~0xFFF
                except:
                    continue
                
                # Check next few instructions for ADD to same register
                adrp_reg = ops[0].strip()
                for j in range(i+1, min(i+4, len(insns))):
                    if insns[j].mnemonic == 'add':
                        add_ops = insns[j].op_str.split(',')
                        if len(add_ops) >= 3 and add_ops[1].strip() == adrp_reg:
                            try:
                                offset = int(add_ops[2].strip().replace('#', ''), 16)
                                full_addr = page_val + offset
                                file_offset = full_addr - ROM_BASE
                                
                                # Check if this points to any of our strings
                                for str_addr, str_val in strings_found.items():
                                    if abs(file_offset - str_addr) <= 2:
                                        if str_addr not in string_refs:
                                            string_refs[str_addr] = []
                                        string_refs[str_addr].append({
                                            'code_addr': insns[i].address,
                                            'adrp': f"{insns[i].mnemonic} {insns[i].op_str}",
                                            'add': f"{insns[j].mnemonic} {insns[j].op_str}",
                                            'computed': full_addr,
                                        })
                            except:
                                pass
    
    print(f"\n  Code references to USB/DFU strings:")
    for str_addr, refs in sorted(string_refs.items()):
        s = strings_found.get(str_addr, "?")
        print(f"\n    String \"{s}\" @ 0x{str_addr:05X}:")
        for ref in refs:
            print(f"      Referenced from 0x{ref['code_addr']:X}: {ref['adrp']} + {ref['add']}")
            
            # Find which function this reference is in
            func_start = None
            ref_offset = ref['code_addr'] - ROM_BASE
            for off in range(ref_offset, max(ref_offset - 0x1000, 0), -4):
                word = struct.unpack('<I', data[off:off+4])[0]
                # Check for STP x29, x30 pattern (function prologue)
                if (word & 0xFFC07FFF) == 0xA9007BFD:  # stp x29, x30, [sp, #imm]
                    func_start = ROM_BASE + off
                    break
                if (word & 0xFFE0FFFF) == 0xA9BF7BFD:  # stp x29, x30, [sp, #-imm]!
                    func_start = ROM_BASE + off
                    break
            if func_start:
                print(f"      In function starting at 0x{func_start:X}")
    
    # Step 3: Find USB MMIO patterns
    # Apple USB controller registers are typically at 0x39000000 or similar
    print(f"\n  USB controller MMIO references:")
    usb_mmio_refs = []
    for i in range(len(insns) - 1):
        if insns[i].mnemonic == 'mov' or insns[i].mnemonic == 'movk':
            ops = insns[i].op_str
            # Look for common USB OTG base addresses
            for pattern in ['0x3900', '0x3910', '0x3920']:
                if pattern in ops:
                    usb_mmio_refs.append((insns[i].address, f"{insns[i].mnemonic} {insns[i].op_str}"))
    
    for addr, insn_str in usb_mmio_refs[:30]:
        print(f"    0x{addr:X}: {insn_str}")
    
    # Step 4: Find USB setup packet processing
    # USB control transfers have 8-byte setup packets with:
    # bmRequestType (1 byte), bRequest (1 byte), wValue (2), wIndex (2), wLength (2)
    # Standard Apple DFU requests: 1=DNLOAD, 2=UPLOAD, 3=GETSTATUS, 4=CLRSTATUS, 5=GETSTATE, 6=ABORT
    print(f"\n  Looking for USB request type dispatching (DFU class requests):")
    print(f"  DFU requests: DNLOAD=1, UPLOAD=2, GETSTATUS=3, CLRSTATUS=4, GETSTATE=5, ABORT=6")
    
    for i in range(len(insns) - 3):
        # Look for: cmp wN, #1 ... cmp wN, #2 ... etc (request dispatch)
        if insns[i].mnemonic == 'cmp':
            ops = insns[i].op_str.split(',')
            if len(ops) >= 2:
                try:
                    val = int(ops[1].strip().replace('#', ''), 0)
                except:
                    continue
                
                if val in (1, 2, 3, 4, 5, 6):
                    reg = ops[0].strip()
                    # Check if nearby instructions compare same register against other DFU values
                    dfu_vals_found = {val}
                    for j in range(max(i-10, 0), min(i+10, len(insns))):
                        if insns[j].mnemonic == 'cmp' and j != i:
                            cmp_ops = insns[j].op_str.split(',')
                            if len(cmp_ops) >= 2 and cmp_ops[0].strip() == reg:
                                try:
                                    v = int(cmp_ops[1].strip().replace('#', ''), 0)
                                    if v in (1, 2, 3, 4, 5, 6):
                                        dfu_vals_found.add(v)
                                except:
                                    pass
                    
                    if len(dfu_vals_found) >= 3:
                        print(f"\n    POTENTIAL DFU REQUEST DISPATCH @ 0x{insns[i].address:X}")
                        print(f"    Register: {reg}, DFU values found: {sorted(dfu_vals_found)}")
                        # Print context
                        for j in range(max(i-5, 0), min(i+15, len(insns))):
                            marker = " >>>" if j == i else "    "
                            print(f"    {marker} 0x{insns[j].address:X}: {insns[j].mnemonic} {insns[j].op_str}")
    
    return string_refs


# =====================================================================================================================
# HUNT 3: HEAP MANAGEMENT (checkm8 was use-after-free in USB I/O buffer)
# Find malloc/free/memalign and their callers, especially in USB context
# =====================================================================================================================
def hunt_heap_uaf(data, rom_name="B1"):
    print(f"\n{'='*120}")
    print(f"  HUNT 3: HEAP MANAGEMENT & USE-AFTER-FREE PATTERNS IN {rom_name}")
    print(f"{'='*120}\n")
    
    # Find malloc by the "malloc() returns NULL" string
    malloc_str = data.find(b'malloc() returns NULL')
    print(f"  'malloc() returns NULL' string at offset 0x{malloc_str:X}")
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    code = data[:0x24B40]
    insns = list(md.disasm(code, ROM_BASE))
    
    # Find all BL targets (call graph)
    call_graph = {}  # callee -> [caller_addrs]
    for insn in insns:
        if insn.mnemonic == 'bl':
            try:
                target = int(insn.op_str.replace('#', ''), 16)
                if target not in call_graph:
                    call_graph[target] = []
                call_graph[target].append(insn.address)
            except:
                pass
    
    # Find memalign/malloc by pattern:
    # malloc typically: takes size in x0, returns pointer in x0
    # memalign: takes alignment in x0, size in x1
    # free: takes pointer in x0
    
    # Look for functions that reference the malloc string
    malloc_str_page = (ROM_BASE + malloc_str) & ~0xFFF
    malloc_str_offset = (ROM_BASE + malloc_str) & 0xFFF
    
    malloc_func = None
    free_func = None
    
    for i in range(len(insns) - 2):
        if insns[i].mnemonic == 'adrp':
            ops = insns[i].op_str.split(',')
            if len(ops) >= 2:
                try:
                    page = int(ops[1].strip().replace('#', ''), 16)
                    if page == malloc_str_page:
                        # Check ADD for exact offset
                        reg = ops[0].strip()
                        for j in range(i+1, min(i+4, len(insns))):
                            if insns[j].mnemonic == 'add' and reg in insns[j].op_str:
                                add_ops = insns[j].op_str.split(',')
                                if len(add_ops) >= 3:
                                    try:
                                        off = int(add_ops[2].strip().replace('#', ''), 16)
                                        if off == malloc_str_offset:
                                            print(f"  malloc string ref at 0x{insns[i].address:X}")
                                            # This is in the panic/error path of malloc
                                            # Find the function start
                                            ref_off = insns[i].address - ROM_BASE
                                            for off2 in range(ref_off, max(ref_off - 0x200, 0), -4):
                                                w = struct.unpack('<I', data[off2:off2+4])[0]
                                                if (w & 0xFFC07FFF) == 0xA9007BFD or (w & 0xFFE0FFFF) == 0xA9BF7BFD:
                                                    malloc_func = ROM_BASE + off2
                                                    print(f"  malloc function at 0x{malloc_func:X}")
                                                    break
                                    except:
                                        pass
                except:
                    pass
    
    # Find free: typically small function that takes pointer in x0
    # Free often preceded by cbz x0 (null check)
    # Or find by looking for what's called right before memory reuse patterns
    
    # Method: find function that does: cbz x0, ... <free logic>
    # Common free pattern: load header, mark as free, coalesce
    
    # Find functions called frequently (top callees) - malloc/free are among the most called
    print(f"\n  Top 20 most-called functions (likely includes malloc/free/memcpy):")
    sorted_callees = sorted(call_graph.items(), key=lambda x: len(x[1]), reverse=True)
    for callee, callers in sorted_callees[:20]:
        # Disassemble first few instructions to identify
        off = callee - ROM_BASE
        if 0 <= off < len(data) - 20:
            func_insns = disasm_function(data, off, max_insns=20)
            first_insns = ' ; '.join([f"{i.mnemonic} {i.op_str}" for i in func_insns[:4]])
            print(f"    0x{callee:X} called {len(callers):3d} times: {first_insns}")
    
    # Now look for use-after-free patterns:
    # Pattern: bl <free> ... (intervening code) ... ldr/str [freed_pointer]
    # Or: register holding freed pointer used after free
    
    print(f"\n  Analyzing potential use-after-free patterns around most-called functions...")
    
    # If we can identify malloc and free, look for:
    # 1. Pointer allocated with malloc
    # 2. Pointer freed
    # 3. Pointer used again without re-validation
    
    # For now, look for a simpler pattern: BL followed by CBZ (null check on alloc)
    # then the allocated pointer used, then BL, then same pointer used again
    
    # More useful: find all "free-like" functions (called with pointer, no return value used)
    # vs "malloc-like" (return value stored and used)
    
    return call_graph


# =====================================================================================================================
# HUNT 4: BOUNDS CHECK ANALYSIS
# Look for memory copy/transfer operations without proper bounds checking
# Especially: memcpy with user-influenced length, buffer operations without size validation
# =====================================================================================================================
def hunt_bounds_checks(data, rom_name="B1"):
    print(f"\n{'='*120}")
    print(f"  HUNT 4: BOUNDS CHECK ANALYSIS IN {rom_name}")
    print(f"  Looking for memory operations without proper size validation")
    print(f"{'='*120}\n")
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    code = data[:0x24B40]
    insns = list(md.disasm(code, ROM_BASE))
    
    # Find memcpy/memmove by pattern:
    # These typically have a loop with LDR/STR and a counter
    # Or they use NEON bulk copy (LD1/ST1)
    
    # Look for memcpy string
    memcpy_str = data.find(b'memcpy')
    memmove_str = data.find(b'memmove')
    print(f"  'memcpy' string at offset: 0x{memcpy_str:X}" if memcpy_str != -1 else "  'memcpy' string: NOT FOUND")
    print(f"  'memmove' string at offset: 0x{memmove_str:X}" if memmove_str != -1 else "  'memmove' string: NOT FOUND")
    
    # Pattern: Look for CBZ on size before memcpy-like call
    # Missing size validation before BL could indicate a bug
    
    # Find all BL calls that are preceded by a size argument in x2 (memcpy convention: dst=x0, src=x1, len=x2)
    # Check if x2 is validated (compared against a bound) before the BL
    
    print(f"\n  Looking for BL calls where x2 (length) comes from memory without bounds check...")
    
    suspicious = []
    for i in range(len(insns)):
        if insns[i].mnemonic == 'bl':
            target = insns[i].op_str
            
            # Look backwards for x2 being set from memory (user data)
            x2_from_mem = False
            x2_checked = False
            x2_load_addr = None
            
            for j in range(i-1, max(i-15, 0), -1):
                # x2 loaded from memory
                if insns[j].mnemonic == 'ldr' and 'w2,' in insns[j].op_str or 'x2,' in insns[j].op_str:
                    if '[' in insns[j].op_str:  # memory load
                        x2_from_mem = True
                        x2_load_addr = insns[j].address
                
                # x2 compared (bounds check)
                if insns[j].mnemonic == 'cmp' and ('w2' in insns[j].op_str or 'x2' in insns[j].op_str):
                    x2_checked = True
                
                # x2 set from register (might be validated elsewhere)
                if insns[j].mnemonic == 'mov' and insns[j].op_str.startswith(('x2,', 'w2,')):
                    break
            
            if x2_from_mem and not x2_checked:
                suspicious.append({
                    'bl_addr': insns[i].address,
                    'bl_target': target,
                    'load_addr': x2_load_addr,
                })
    
    print(f"  Found {len(suspicious)} BL calls with unchecked memory-loaded x2:")
    for s in suspicious[:20]:
        print(f"    0x{s['bl_addr']:X}: bl {s['bl_target']} (x2 loaded from mem at 0x{s['load_addr']:X})")
    
    # Look for buffer overflow patterns:
    # STR to [base, offset] where offset could exceed buffer size
    # Especially in loops
    
    print(f"\n  Looking for indexed stores in loops (potential buffer overflow)...")
    loop_stores = []
    for i in range(len(insns)):
        if insns[i].mnemonic in ('str', 'stp') and '[' in insns[i].op_str:
            # Check if this is in a loop (backwards branch nearby)
            in_loop = False
            for j in range(i+1, min(i+30, len(insns))):
                if insns[j].mnemonic in ('b.ne', 'b.lt', 'b.lo', 'b.le', 'b.ls', 'cbnz'):
                    try:
                        target_str = insns[j].op_str.split('#')[-1]
                        target = int(target_str, 16)
                        if target <= insns[i].address:
                            in_loop = True
                            break
                    except:
                        pass
            
            if in_loop:
                loop_stores.append(insns[i])
    
    print(f"  Found {len(loop_stores)} STR/STP instructions inside loops")
    # Group by function
    seen_funcs = set()
    for insn in loop_stores[:50]:
        off = insn.address - ROM_BASE
        # Find function start
        for back in range(off, max(off-0x500, 0), -4):
            w = struct.unpack('<I', data[back:back+4])[0]
            if (w & 0xFFC07FFF) == 0xA9007BFD or (w & 0xFFE0FFFF) == 0xA9BF7BFD:
                func = ROM_BASE + back
                if func not in seen_funcs:
                    seen_funcs.add(func)
                    print(f"    Function 0x{func:X}: loop with stores (e.g., 0x{insn.address:X}: {insn.mnemonic} {insn.op_str})")
                break
    
    return suspicious


# =====================================================================================================================
# HUNT 5: WHAT'S DIFFERENT IN img4_verify (B1 has 150 insns vs A0's 77)
# The 73 extra instructions are Apple's expanded signature verification
# =====================================================================================================================
def hunt_img4_expansion(data_a0, data_b1):
    print(f"\n{'='*120}")
    print(f"  HUNT 5: img4_verify EXPANSION ANALYSIS")
    print(f"  A0: 77 instructions @ 0x10000A3A0")
    print(f"  B1: 150 instructions @ 0x10000A704")
    print(f"  What did Apple ADD?")
    print(f"{'='*120}\n")
    
    # Disassemble both
    a0_insns = disasm_function(data_a0, 0xA3A0, max_insns=200)
    b1_insns = disasm_function(data_b1, 0xA704, max_insns=200)
    
    # But img4_verify doesn't end at first RET in B1 - it's more complex
    # Let's disassemble more
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    
    # A0: 77 instructions from 0xA3A0
    a0_code = data_a0[0xA3A0:0xA3A0 + 77*4 + 100]
    a0_all = list(md.disasm(a0_code, ROM_BASE + 0xA3A0))[:80]
    
    # B1: 150 instructions from 0xA704
    b1_code = data_b1[0xA704:0xA704 + 200*4]
    b1_all = list(md.disasm(b1_code, ROM_BASE + 0xA704))[:160]
    
    # Find the crypto_verify call in both (the deep verification)
    print(f"  A0 img4_verify calls (BL instructions):")
    for insn in a0_all:
        if insn.mnemonic == 'bl':
            print(f"    0x{insn.address:X}: bl {insn.op_str}")
    
    print(f"\n  B1 img4_verify calls (BL instructions):")
    for insn in b1_all:
        if insn.mnemonic == 'bl':
            print(f"    0x{insn.address:X}: bl {insn.op_str}")
    
    # Compare branch structure
    print(f"\n  A0 conditional branches:")
    for insn in a0_all:
        if insn.mnemonic.startswith('b.') or insn.mnemonic in ('cbz', 'cbnz', 'tbz', 'tbnz'):
            print(f"    0x{insn.address:X}: {insn.mnemonic} {insn.op_str}")
    
    print(f"\n  B1 conditional branches:")
    for insn in b1_all:
        if insn.mnemonic.startswith('b.') or insn.mnemonic in ('cbz', 'cbnz', 'tbz', 'tbnz'):
            print(f"    0x{insn.address:X}: {insn.mnemonic} {insn.op_str}")
    
    # Key question: what's the EXTRA code in B1?
    # Compare the structure: prologue, main logic, error handling, epilogue
    
    # Count unique call targets
    a0_calls = set()
    b1_calls = set()
    for insn in a0_all:
        if insn.mnemonic == 'bl':
            a0_calls.add(insn.op_str)
    for insn in b1_all:
        if insn.mnemonic == 'bl':
            b1_calls.add(insn.op_str)
    
    print(f"\n  A0 unique call targets: {len(a0_calls)}")
    print(f"  B1 unique call targets: {len(b1_calls)}")
    print(f"  New calls in B1: {b1_calls - a0_calls}" if b1_calls - a0_calls else "  No new unique calls")
    
    # Analyze B1's extra prologue
    print(f"\n  B1 prologue analysis (register saves):")
    for insn in b1_all[:15]:
        print(f"    0x{insn.address:X}: {insn.mnemonic} {insn.op_str}")
    
    print(f"\n  A0 prologue analysis:")
    for insn in a0_all[:10]:
        print(f"    0x{insn.address:X}: {insn.mnemonic} {insn.op_str}")


# =====================================================================================================================
# HUNT 6: RACE CONDITIONS & STATE MACHINE
# The DFU state machine has states. Transitions without proper locking = race
# checkm8 exploited a race in USB reset during DFU DNLOAD
# =====================================================================================================================
def hunt_state_machine(data, rom_name="B1"):
    print(f"\n{'='*120}")
    print(f"  HUNT 6: STATE MACHINE & RACE CONDITION PATTERNS IN {rom_name}")
    print(f"{'='*120}\n")
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    code = data[:0x24B40]
    insns = list(md.disasm(code, ROM_BASE))
    
    # State machines typically use: ldr wN, [state_ptr] ; cmp wN, #state ; b.xx
    # Look for global state variables (loaded via ADRP+LDR pattern, then compared against constants)
    
    state_patterns = []
    for i in range(len(insns) - 5):
        if insns[i].mnemonic == 'adrp':
            adrp_ops = insns[i].op_str.split(',')
            if len(adrp_ops) < 2:
                continue
            reg = adrp_ops[0].strip()
            
            # Look for LDR from this register (state read)
            for j in range(i+1, min(i+4, len(insns))):
                if insns[j].mnemonic == 'ldr' and reg in insns[j].op_str:
                    load_reg = insns[j].op_str.split(',')[0].strip()
                    
                    # Look for CMP against constants (state check)
                    for k in range(j+1, min(j+5, len(insns))):
                        if insns[k].mnemonic == 'cmp' and load_reg in insns[k].op_str:
                            cmp_ops = insns[k].op_str.split(',')
                            if len(cmp_ops) >= 2 and '#' in cmp_ops[1]:
                                try:
                                    state_val = int(cmp_ops[1].strip().replace('#', ''), 0)
                                    if state_val <= 10:  # Small constants = state values
                                        state_patterns.append({
                                            'addr': insns[i].address,
                                            'adrp': insns[i].op_str,
                                            'load': f"{insns[j].mnemonic} {insns[j].op_str}",
                                            'cmp': f"{insns[k].mnemonic} {insns[k].op_str}",
                                            'state_val': state_val,
                                        })
                                except:
                                    pass
                    break
    
    # Group by ADRP target (same state variable)
    from collections import defaultdict
    state_vars = defaultdict(list)
    for sp in state_patterns:
        key = sp['adrp'].split(',')[1].strip()
        state_vars[key].append(sp)
    
    print(f"  Found {len(state_vars)} potential state variables:")
    for var, patterns in sorted(state_vars.items(), key=lambda x: -len(x[1])):
        states = set(p['state_val'] for p in patterns)
        print(f"\n    State var page {var} ({len(patterns)} references, states: {sorted(states)}):")
        for p in patterns[:8]:
            print(f"      0x{p['addr']:X}: {p['adrp']} → {p['load']} → {p['cmp']}")
    
    # Look for state WRITES without proper ordering
    # Pattern: STR to state variable without DSB/DMB barrier
    print(f"\n  Looking for state writes without memory barriers...")
    
    barrier_insns = set()
    for insn in insns:
        if insn.mnemonic in ('dsb', 'dmb', 'isb'):
            barrier_insns.add(insn.address)
    
    print(f"  Total memory barriers in ROM: {len(barrier_insns)}")
    
    return state_vars


# =====================================================================================================================
# HUNT 7: THE CRITICAL 32-BIT COMPARISON — DEEP ANALYSIS
# In A0: cmp w2, w20 at 0x100001A44 | In B1: cmp x2, x24 at 0x100001B70
# What exactly is being compared? Is the fix complete?
# =====================================================================================================================
def hunt_32bit_comparison_context(data_a0, data_b1):
    print(f"\n{'='*120}")
    print(f"  HUNT 7: THE 32-BIT COMPARISON BUG — DEEP CONTEXT ANALYSIS")
    print(f"  A0: cmp w2, w20 (32-bit) | B1: cmp x2, x24 (64-bit)")
    print(f"  What is being compared and where does the data come from?")
    print(f"{'='*120}\n")
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    
    # A0: Context around 0x100001A44 (file offset 0x1A44)
    print(f"  === A0 CONTEXT around cmp w2, w20 @ 0x100001A44 ===")
    a0_code = data_a0[0x19F0:0x1B00]
    a0_insns = list(md.disasm(a0_code, ROM_BASE + 0x19F0))
    for insn in a0_insns:
        marker = " >>>" if insn.address == 0x100001A44 else "    "
        print(f"  {marker} 0x{insn.address:X}: {insn.mnemonic:8s} {insn.op_str}")
    
    # B1: Context around 0x100001B70 (file offset 0x1B70)
    print(f"\n  === B1 CONTEXT around cmp x2, x24 @ 0x100001B70 ===")
    b1_code = data_b1[0x1B14:0x1C30]
    b1_insns = list(md.disasm(b1_code, ROM_BASE + 0x1B14))
    for insn in b1_insns:
        marker = " >>>" if insn.address == 0x100001B70 else "    "
        print(f"  {marker} 0x{insn.address:X}: {insn.mnemonic:8s} {insn.op_str}")
    
    # Trace where w20/x24 comes from (the comparison target / expected size)
    print(f"\n  === Tracing origin of the comparison target ===")
    
    # In A0, w20 is set somewhere earlier in the boot function
    print(f"  A0: Where is w20 set (the expected size)?")
    a0_boot = data_a0[0x17A4:0x1B80]
    a0_boot_insns = list(md.disasm(a0_boot, ROM_BASE + 0x17A4))
    for insn in a0_boot_insns:
        if 'w20' in insn.op_str.split(',')[0] or 'x20' in insn.op_str.split(',')[0]:
            if insn.mnemonic not in ('stp', 'ldp') or 'w20' == insn.op_str.split(',')[0].strip():
                print(f"    0x{insn.address:X}: {insn.mnemonic} {insn.op_str}")
    
    print(f"\n  B1: Where is x24 set (the validated size)?")
    b1_boot = data_b1[0x17A4:0x1D00]
    b1_boot_insns = list(md.disasm(b1_boot, ROM_BASE + 0x17A4))
    for insn in b1_boot_insns:
        if 'w24' in insn.op_str.split(',')[0] or 'x24' in insn.op_str.split(',')[0]:
            if insn.mnemonic not in ('stp', 'ldp') or 'x24' == insn.op_str.split(',')[0].strip():
                print(f"    0x{insn.address:X}: {insn.mnemonic} {insn.op_str}")


# =====================================================================================================================
# MAIN
# =====================================================================================================================
if __name__ == '__main__':
    import sys
    
    print("ROM VULNERABILITY HUNTER")
    print("=" * 120)
    
    data_b1 = load_rom('securerom/t8020_B1_securerom.bin')
    data_a0 = load_rom('securerom/t8020_A0_securerom.bin')
    
    # Run all hunts on B1 (our target)
    trunc_findings = hunt_integer_truncation(data_b1, "B1")
    string_refs = hunt_dfu_handler(data_b1, "B1")
    call_graph = hunt_heap_uaf(data_b1, "B1")
    bounds_findings = hunt_bounds_checks(data_b1, "B1")
    hunt_img4_expansion(data_a0, data_b1)
    state_vars = hunt_state_machine(data_b1, "B1")
    hunt_32bit_comparison_context(data_a0, data_b1)
    
    # Also check A0 for comparison
    print(f"\n\n{'#'*120}")
    print(f"# CROSS-CHECK: Integer truncation in A0 (to confirm the bug Apple fixed)")
    print(f"{'#'*120}")
    a0_trunc = hunt_integer_truncation(data_a0, "A0")
    
    print(f"\n\n{'='*120}")
    print(f"  VULNERABILITY HUNT COMPLETE")
    print(f"{'='*120}")
