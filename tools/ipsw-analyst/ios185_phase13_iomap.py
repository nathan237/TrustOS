#!/usr/bin/env python3
"""
Phase 13: Find ml_io_map, alternative physical read paths, and A13 memory map
iOS 18.5 (22F76) kernelcache analysis

Goals:
1. Locate ml_io_map / ml_io_unmap (MMIO mapping — no gPhysBase check)
2. Find any physical read functions WITHOUT bounds checks
3. Locate DART/IOMMU functions for device memory mapping
4. Search for device tree entries revealing A13 physical memory layout
5. Find copyin/copyout and other kernel memory primitives
"""

import struct
import os
import json
from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM

KC_PATH = "extracted/kernelcache_iPhone12,3_18_5.raw"
KC_BASE = 0xfffffff007004000
OUTPUT = "phase13_out.txt"

# Known addresses
G_PHYS_BASE = 0xfffffff007a6cbb8
G_PHYS_SIZE = 0xfffffff007a6cbc0
ML_PHYS_READ_CORE = 0xfffffff00807b4f8
ML_PHYS_WRITE_CORE = 0xfffffff00807b7a8

def va_to_off(va):
    return va - KC_BASE

def off_to_va(off):
    return off + KC_BASE

def main():
    print(f"[*] Loading kernelcache from {KC_PATH}")
    with open(KC_PATH, 'rb') as f:
        kc = f.read()
    print(f"[+] Loaded {len(kc):,} bytes")
    
    out_lines = []
    def log(msg=""):
        print(msg)
        out_lines.append(msg)
    
    cs = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    cs.detail = True
    
    # ═══════════════════════════════════════════════════════════════
    # 1. Find ml_io_map string and trace to function
    # ═══════════════════════════════════════════════════════════════
    log("=" * 70)
    log(" PHASE 13A: Locate ml_io_map / ml_io_unmap")
    log("=" * 70)
    
    target_strings = [
        b"ml_io_map",
        b"ml_io_unmap", 
        b"io_map",
        b"io_unmap",
        b"ml_static_mfree",
        b"ml_static_malloc",
        b"mmio",
        b"IOMemoryMap",
        b"mapDeviceMemory",
        b"iokit_mem",
    ]
    
    for target in target_strings:
        log(f"\n--- Searching for string: '{target.decode('utf-8', errors='replace')}' ---")
        pos = 0
        found_count = 0
        while True:
            idx = kc.find(target, pos)
            if idx == -1:
                break
            # Make sure it's a null-terminated C string match (not substring of larger)
            va = off_to_va(idx)
            # Check bytes around
            pre = kc[max(0,idx-1):idx]
            post_byte = kc[idx+len(target):idx+len(target)+1] if idx+len(target) < len(kc) else b''
            
            context = kc[max(0,idx-4):min(len(kc),idx+len(target)+16)]
            log(f"  Found @ offset 0x{idx:x} (VA 0x{va:016x})")
            log(f"    Context: {context}")
            
            # If it looks like a proper string start (preceded by null or is a word boundary)
            if pre == b'\x00' or pre == b'' or idx == 0:
                log(f"    → Likely string start (preceded by null)")
                
                # Search for ADRP+ADD referencing this address within ~256MB
                page = va & ~0xFFF
                page_off = va & 0xFFF
                found_refs = find_adrp_add_refs(kc, cs, va, page, page_off, log)
                if found_refs:
                    log(f"    → Found {len(found_refs)} xref(s)")
            
            found_count += 1
            pos = idx + 1
            if found_count >= 10:
                log(f"    (... truncated, {found_count}+ matches)")
                break
        
        if found_count == 0:
            log(f"  NOT FOUND")
    
    # ═══════════════════════════════════════════════════════════════
    # 2. Search for physical read functions without gPhysBase check
    # ═══════════════════════════════════════════════════════════════
    log("\n" + "=" * 70)
    log(" PHASE 13B: Find unchecked physical read primitives")
    log("=" * 70)
    
    # Look for functions near ml_phys_read that DON'T reference gPhysBase
    # The region around 0x807b000-0x807c000 has the phys read/write cluster
    region_start_va = ML_PHYS_READ_CORE - 0x400
    region_end_va = ML_PHYS_WRITE_CORE + 0x800
    region_start_off = va_to_off(region_start_va)
    region_end_off = va_to_off(region_end_va)
    
    log(f"\n--- Scanning region 0x{region_start_va:x} - 0x{region_end_va:x} ---")
    log(f"    (around ml_phys_read/write cluster)")
    
    # Find all function starts (look for STP x29, x30 prologues)
    func_starts = []
    for off in range(region_start_off, min(region_end_off, len(kc)), 4):
        insn_bytes = kc[off:off+4]
        if len(insn_bytes) < 4:
            break
        val = struct.unpack('<I', insn_bytes)[0]
        # STP x29, x30, [sp, #-N]! (pre-index)
        # Encoding: 10 1010 00 11 iiiiiii 11110 11101 11111
        # Mask: STP with x29, x30 to sp
        if (val & 0xFFE07FFF) == 0xA9807BFD:
            func_starts.append(off)
    
    log(f"  Found {len(func_starts)} function prologues in region")
    
    # For each function, check if it references gPhysBase
    gphysbase_page = G_PHYS_BASE & ~0xFFF
    
    for fstart in func_starts:
        fva = off_to_va(fstart)
        # Disassemble up to 512 bytes
        chunk = kc[fstart:fstart+512]
        refs_gphysbase = False
        has_ldr_phys = False
        func_size = 0
        insn_list = []
        
        for insn in cs.disasm(chunk, fva):
            insn_list.append(insn)
            func_size = insn.address - fva + insn.size
            
            # Check for ADRP to gPhysBase page
            if insn.mnemonic == 'adrp':
                # Extract immediate
                ops = insn.operands
                if len(ops) >= 2:
                    imm = ops[1].imm
                    if imm == gphysbase_page:
                        refs_gphysbase = True
            
            # Check for RET
            if insn.mnemonic == 'ret':
                break
        
        has_physmap = False
        has_mrs_tpidr = False
        has_ldr_from_x0 = False
        
        for insn in insn_list:
            mnem = insn.mnemonic
            op = insn.op_str
            
            if 'tpidr_el1' in op.lower():
                has_mrs_tpidr = True
            
            # MOV with physmap constant components
            if mnem in ('mov', 'movz', 'movk') and '0xffd00000' in op:
                has_physmap = True
            
            # LDR/LDRB from x0 (reading from physical-mapped address)
            if mnem.startswith('ldr') and '[x' in op:
                has_ldr_from_x0 = True
        
        status = "BOUNDED" if refs_gphysbase else "*** NO gPhysBase CHECK ***"
        log(f"\n  Function @ 0x{fva:016x} ({func_size} bytes) [{status}]")
        log(f"    refs_gPhysBase: {refs_gphysbase}")
        log(f"    has_physmap_const: {has_physmap}")
        log(f"    has_mrs_tpidr: {has_mrs_tpidr}")
        
        if not refs_gphysbase and (has_physmap or has_mrs_tpidr):
            log(f"    >>> INTERESTING: uses physmap/preempt without bounds check!")
            # Print full disassembly
            for insn in insn_list:
                log(f"      0x{insn.address:x}: {insn.mnemonic} {insn.op_str}")
    
    # ═══════════════════════════════════════════════════════════════
    # 3. Search for copyin/copyout/bcopy_phys
    # ═══════════════════════════════════════════════════════════════
    log("\n" + "=" * 70)
    log(" PHASE 13C: Find bcopy_phys / copyin / copyout")
    log("=" * 70)
    
    phys_strings = [
        b"bcopy_phys",
        b"ml_phys_read_data",  # already found, use as anchor
        b"ml_phys_write_data",
        b"phys_copy",
        b"pmap_enter",
        b"pmap_remove",
        b"kvtophys",
        b"phystokv",
    ]
    
    for target in phys_strings:
        log(f"\n--- String: '{target.decode()}' ---")
        idx = kc.find(target + b'\x00')
        if idx == -1:
            idx = kc.find(target)
        if idx != -1:
            va = off_to_va(idx)
            log(f"  Found @ VA 0x{va:016x} (offset 0x{idx:x})")
            # Quick xref scan
            find_adrp_add_refs(kc, cs, va, va & ~0xFFF, va & 0xFFF, log, max_refs=3)
        else:
            log(f"  NOT FOUND in kernelcache")
    
    # ═══════════════════════════════════════════════════════════════
    # 4. Device tree / memory map entries
    # ═══════════════════════════════════════════════════════════════
    log("\n" + "=" * 70)
    log(" PHASE 13D: A13 Memory Map / Device Tree")
    log("=" * 70)
    
    # Search for known A13/T8030 memory map strings
    dt_strings = [
        b"t8030",
        b"T8030",
        b"SecureROM",
        b"BootROM",
        b"SRAM",
        b"boot-rom",
        b"secure-rom",
        b"rom-region",
        b"memory-map",
        b"dram-base",
        b"phys-mem",
        b"reg-private",
    ]
    
    for target in dt_strings:
        log(f"\n--- String: '{target.decode('utf-8', errors='replace')}' ---")
        pos = 0
        count = 0
        while count < 5:
            idx = kc.find(target, pos)
            if idx == -1:
                break
            va = off_to_va(idx)
            context = kc[max(0,idx-8):min(len(kc),idx+len(target)+32)]
            log(f"  @ VA 0x{va:016x}: {context}")
            pos = idx + 1
            count += 1
        if count == 0:
            log(f"  NOT FOUND")
    
    # ═══════════════════════════════════════════════════════════════
    # 5. Look for IOMemoryDescriptor::createMappingInTask patterns
    #    (kernel API for mapping physical memory to VA)
    # ═══════════════════════════════════════════════════════════════
    log("\n" + "=" * 70)
    log(" PHASE 13E: IOMemoryDescriptor mapping functions")
    log("=" * 70)
    
    io_strings = [
        b"IOMemoryDescriptor",
        b"createMappingInTask",
        b"IODeviceMemory",
        b"getPhysicalAddress",
        b"getVirtualAddress",
        b"withPhysicalAddress",
        b"IOMapPages",
    ]
    
    for target in io_strings:
        log(f"\n--- String: '{target.decode()}' ---")
        pos = 0
        count = 0
        while count < 5:
            idx = kc.find(target, pos)
            if idx == -1:
                break
            va = off_to_va(idx)
            context_end = min(len(kc), idx + len(target) + 64)
            # Read until null
            end = idx + len(target)
            while end < context_end and kc[end] != 0:
                end += 1
            full_str = kc[idx:end]
            log(f"  @ VA 0x{va:016x}: {full_str.decode('utf-8', errors='replace')}")
            pos = idx + 1
            count += 1
        if count == 0:
            log(f"  NOT FOUND")
    
    # ═══════════════════════════════════════════════════════════════
    # 6. Deep scan: functions that load physmap_base WITHOUT gPhysBase
    # ═══════════════════════════════════════════════════════════════
    log("\n" + "=" * 70)
    log(" PHASE 13F: Physmap-direct functions (no bounds check)")
    log("=" * 70)
    
    # Search for the physmap MOV pattern: MOV Xn, #0xffd00000
    # Encoding for MOV x9, #0xffd00000 = MOVZ x9, #0xd000, LSL#16 ; MOVK x9, #0xfff, LSL#16
    # Actually: look for the 0xfffffffbffd00000 construction sequence
    # MOVZ: 0xffd00000 → MOVZ Xd, #imm16, LSL#16 not exactly standard
    # From phase11c: MOV x9, #0xffd00000 then MOVK x9, #0xfffb, LSL#32 then MOVK x9, #0xffff, LSL#48
    
    # Search for MOVK with #0xfffb, LSL#32 (part of physmap_base)
    # MOVK x?, #0xfffb, LSL#32 
    # Encoding: 1 11 100101 10 (hw=2) imm16=0xfffb Rd
    # = 0xF2C0_0000 | (0xFFFB << 5) | Rd
    # = 0xF2C0_0000 | 0x1FFF60 | Rd
    # = 0xF2DFFF60 | Rd
    
    movk_pattern = 0xF2DFFF60  # MOVK x0, #0xfffb, LSL#32 (Rd=0)
    log(f"\n--- Scanning for MOVK Xn, #0xfffb, LSL#32 (physmap_base component) ---")
    
    physmap_funcs = []
    for off in range(0, len(kc) - 4, 4):
        val = struct.unpack('<I', kc[off:off+4])[0]
        if (val & 0xFFFFFFE0) == movk_pattern:
            rd = val & 0x1F
            va = off_to_va(off)
            # Find function start (scan backwards for STP prologue)
            func_start = find_func_start(kc, off)
            func_va = off_to_va(func_start) if func_start else 0
            
            # Check if this function also references gPhysBase
            refs_gphys = False
            if func_start:
                func_chunk = kc[func_start:func_start+1024]
                for insn in cs.disasm(func_chunk, func_va):
                    if insn.mnemonic == 'adrp' and len(insn.operands) >= 2:
                        if insn.operands[1].imm == gphysbase_page:
                            refs_gphys = True
                            break
                    if insn.mnemonic == 'ret':
                        break
            
            tag = "BOUNDED" if refs_gphys else "NO-CHECK"
            physmap_funcs.append((va, func_va, rd, refs_gphys))
            log(f"  MOVK x{rd}, #0xfffb, LSL#32 @ 0x{va:016x}  func=0x{func_va:016x} [{tag}]")
    
    log(f"\n  Total physmap_base references: {len(physmap_funcs)}")
    log(f"  Without gPhysBase check: {sum(1 for _,_,_,g in physmap_funcs if not g)}")
    
    # For unchecked ones, dump full function
    unbounded = [(va, fva, rd) for va, fva, rd, g in physmap_funcs if not g and fva != 0]
    if unbounded:
        log(f"\n--- Dumping {len(unbounded)} unbounded physmap functions ---")
        seen_func = set()
        for mva, fva, rd in unbounded:
            if fva in seen_func:
                continue
            seen_func.add(fva)
            foff = va_to_off(fva)
            chunk = kc[foff:foff+1024]
            log(f"\n  === Function @ 0x{fva:016x} ===")
            for insn in cs.disasm(chunk, fva):
                log(f"    0x{insn.address:x}: {insn.mnemonic} {insn.op_str}")
                if insn.mnemonic == 'ret':
                    break
    
    # ═══════════════════════════════════════════════════════════════
    # 7. Find phystokv (physical to kernel virtual translation)
    # ═══════════════════════════════════════════════════════════════
    log("\n" + "=" * 70)
    log(" PHASE 13G: phystokv / kvtophys functions")
    log("=" * 70)
    
    # phystokv: takes physical address, returns kernel VA
    # It likely uses physmap_base + (phys - gPhysBase)
    # But there might be an unchecked version or one that handles MMIO ranges
    
    # Look for functions right before/after ml_phys_read that are small
    # and do the phys→kv translation
    for search_name, search_bytes in [
        ("phystokv", b"phystokv"),
        ("kvtophys", b"kvtophys"),
        ("ml_static_vtop", b"ml_static_vtop"),
        ("pmap_find_phys", b"pmap_find_phys"),
    ]:
        log(f"\n--- String: '{search_name}' ---")
        idx = kc.find(search_bytes + b'\x00')
        if idx == -1:
            idx = kc.find(search_bytes)
        if idx != -1:
            va = off_to_va(idx)
            log(f"  Found @ VA 0x{va:016x}")
            refs = find_adrp_add_refs(kc, cs, va, va & ~0xFFF, va & 0xFFF, log, max_refs=5)
        else:
            log(f"  NOT FOUND")
    
    # Save output
    log("\n" + "=" * 70)
    log(" PHASE 13 COMPLETE")
    log("=" * 70)
    
    with open(OUTPUT, 'w', encoding='utf-8') as f:
        f.write('\n'.join(out_lines))
    print(f"\n[+] Output saved to {OUTPUT} ({len(out_lines)} lines)")


def find_func_start(kc, off, max_back=4096):
    """Scan backwards to find function prologue (STP x29, x30, [sp, #-N]!)"""
    for scan_off in range(off, max(0, off - max_back), -4):
        if scan_off + 4 > len(kc):
            continue
        val = struct.unpack('<I', kc[scan_off:scan_off+4])[0]
        # STP x29, x30, [sp, #-N]! (pre-indexed)
        if (val & 0xFFE07FFF) == 0xA9807BFD:
            return scan_off
        # Also check for PACIBSP (hint #27) as function start
        if val == 0xD503237F:
            return scan_off
    return None


def find_adrp_add_refs(kc, cs, string_va, page, page_off, log, max_refs=5):
    """Find ADRP+ADD sequences that reference a given VA"""
    found = []
    
    # We need to search a wide range. ADRP can reach ±4GB.
    # Focus on __TEXT_EXEC regions. Scan in chunks.
    # For efficiency, search for the page offset in ADD instructions
    
    # ADD Xd, Xn, #page_off encoding:
    # For 12-bit immediate: 1 00 10001 00 imm12 Rn Rd
    # page_off = va & 0xFFF
    # imm12 = page_off
    
    if page_off > 0xFFF:
        return found
    
    # Encode ADD Xd, Xn, #page_off
    # We can't know Rd/Rn, so search for the imm12 pattern
    # Bits [21:10] = imm12
    add_imm_pattern = (page_off & 0xFFF) << 10
    add_mask = 0x003FFC00  # bits [21:10]
    add_base = 0x91000000  # ADD X (SF=1, op=0, S=0, shift=00)
    add_base_mask = 0xFF000000
    
    # Search entire kernelcache for matching ADD instructions
    for off in range(0, len(kc) - 4, 4):
        val = struct.unpack('<I', kc[off:off+4])[0]
        # Check if this is an ADD with our immediate
        if (val & add_base_mask) == add_base and (val & add_mask) == add_imm_pattern:
            # Check previous instruction for ADRP
            if off >= 4:
                prev_val = struct.unpack('<I', kc[off-4:off])[0]
                # ADRP: 1 immlo 10000 immhi Rd
                if (prev_val & 0x9F000000) == 0x90000000:
                    # Decode ADRP
                    adrp_va = off_to_va(off - 4)
                    rd_adrp = prev_val & 0x1F
                    immhi = (prev_val >> 5) & 0x7FFFF
                    immlo = (prev_val >> 29) & 0x3
                    imm = (immhi << 2) | immlo
                    if imm & 0x100000:  # sign extend 21 bits
                        imm |= ~0x1FFFFF
                        imm &= 0xFFFFFFFFFFFFFFFF
                    adrp_result = (adrp_va & ~0xFFF) + ((imm << 12) & 0xFFFFFFFFFFFFFFFF)
                    adrp_result &= 0xFFFFFFFFFFFFFFFF
                    
                    # Check ADD's Rn matches ADRP's Rd
                    add_rn = (val >> 5) & 0x1F
                    
                    if adrp_result == page and add_rn == rd_adrp:
                        add_va = off_to_va(off)
                        # Find function start
                        func_start = find_func_start(kc, off - 4)
                        func_va = off_to_va(func_start) if func_start else 0
                        
                        log(f"    XREF: ADRP+ADD @ 0x{adrp_va:x} in func 0x{func_va:x}")
                        found.append((adrp_va, func_va))
                        
                        if len(found) >= max_refs:
                            return found
    
    return found


if __name__ == '__main__':
    main()
