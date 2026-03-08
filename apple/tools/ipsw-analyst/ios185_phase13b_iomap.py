#!/usr/bin/env python3
"""
Phase 13b: Deep trace of phystokv, ml_io_map, and IODeviceMemory functions
iOS 18.5 (22F76) — finding bounds-free physical memory access primitives
"""

import struct
import os
from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM

KC_PATH = "extracted/kernelcache_iPhone12,3_18_5.raw"
KC_BASE = 0xfffffff007004000
OUTPUT = "phase13b_out.txt"

def va_to_off(va):
    return va - KC_BASE

def off_to_va(off):
    return off + KC_BASE

def disasm_func(kc, cs, start_va, max_bytes=2048, label=""):
    """Disassemble a function from start_va until RET"""
    lines = []
    off = va_to_off(start_va)
    chunk = kc[off:off+max_bytes]
    if label:
        lines.append(f"\n=== {label} @ 0x{start_va:016x} ===")
    
    insn_count = 0
    for insn in cs.disasm(chunk, start_va):
        lines.append(f"  0x{insn.address:x}: {insn.mnemonic:10s} {insn.op_str}")
        insn_count += 1
        if insn.mnemonic in ('ret', 'retab', 'retaa'):
            break
        if insn.mnemonic == 'b' and not insn.op_str.startswith('#'):
            pass  # conditional
        if insn_count > 500:
            lines.append("  ... (truncated at 500 instructions)")
            break
    
    lines.append(f"  [function size: {insn_count * 4} bytes, {insn_count} instructions]")
    return lines

def find_nlist_symbol(kc, symbol_name):
    """Find a symbol in the LINKEDIT nlist entries by searching for the string"""
    # The symbol table string "_ml_io_map" appears in __LINKEDIT
    # Find it, then search backwards for the nlist entry pointing to it
    target = b"_" + symbol_name.encode() + b"\x00"
    idx = kc.find(target)
    if idx == -1:
        target = symbol_name.encode() + b"\x00"
        idx = kc.find(target)
    if idx == -1:
        return None, None
    
    string_va = off_to_va(idx)
    return idx, string_va

def main():
    print(f"[*] Loading kernelcache")
    with open(KC_PATH, 'rb') as f:
        kc = f.read()
    
    out = []
    def log(msg=""):
        print(msg)
        out.append(msg)
    
    cs = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    cs.detail = True
    
    # ═══════════════════════════════════════════════════════════════
    # 1. Disassemble phystokv (0x80772c8)
    # ═══════════════════════════════════════════════════════════════
    log("=" * 70)
    log(" 1. phystokv @ 0xfffffff0080772c8")
    log("    (converts physical address → kernel virtual address)")
    log("=" * 70)
    
    PHYSTOKV = 0xfffffff0080772c8
    for line in disasm_func(kc, cs, PHYSTOKV, label="phystokv"):
        log(line)
    
    # Also check what references the "phystokv" panic string
    # The panic string is at 0x7058251
    # The xref is at 0x80772f0 in func 0x80772c8
    # So phystokv itself references its own name (for panic)
    
    # ═══════════════════════════════════════════════════════════════
    # 2. Find ml_io_map implementation
    # ═══════════════════════════════════════════════════════════════
    log("\n" + "=" * 70)
    log(" 2. ml_io_map — find implementation")
    log("=" * 70)
    
    # The string "_ml_io_map" was found at 0xa92ebef in __LINKEDIT
    # This is part of the symbol table. Let's find the actual function.
    # 
    # Strategy: search for "ml_io_map" as a panic/assert string in __TEXT
    # to find functions that call themselves "ml_io_map"
    
    ml_io_map_str = b"ml_io_map"
    pos = 0
    while True:
        idx = kc.find(ml_io_map_str, pos)
        if idx == -1:
            break
        va = off_to_va(idx)
        ctx = kc[max(0,idx-16):min(len(kc),idx+32)]
        log(f"  String '{ml_io_map_str.decode()}' @ VA 0x{va:016x}")
        log(f"    Context: {ctx}")
        pos = idx + 1
    
    # Search for __LINKEDIT symbol table entries
    # nlist_64 structure: { uint32_t n_strx; uint8_t n_type; uint8_t n_sect; 
    #                       uint16_t n_desc; uint64_t n_value; }
    # Size: 16 bytes
    # n_value = the function address
    
    # Find "_ml_io_map\0" in string table
    target_sym = b"_ml_io_map\x00"
    idx = kc.find(target_sym)
    if idx != -1:
        log(f"\n  Found '_ml_io_map' symbol string at offset 0x{idx:x}")
        
        # We need to find the LC_SYMTAB to get stroff (string table offset)
        # Parse Mach-O header to find LC_SYMTAB
        # For MH_FILESET, the main header is at offset 0
        
        # Quick: search for nlist entries that have n_value in kernel text range
        # and whose n_strx points to our string
        # 
        # The string table base is hard to determine statically, but we can
        # search for nlist patterns where n_value looks like a kernel address
        
        # Alternative: search backwards from the string for nlist64 structures
        # that reference it. The n_strx is the offset into the string table.
        # 
        # Let's try a different approach: search for commonly exported functions
        # near ml_io_map in the symbol table
        
        # Look at the broader context
        ctx_before = kc[max(0,idx-80):idx]
        ctx_after = kc[idx:min(len(kc),idx+120)]
        log(f"  Context before: {ctx_before}")
        log(f"  Context after: {ctx_after}")
        
        # Parse the symbol table area — look for nearby symbols
        # In __LINKEDIT, symbols are grouped. Let's find the string table range.
        # Strings are null-terminated sequences
        
        # Find the start of this string cluster
        scan = idx
        while scan > 0 and kc[scan-1] != 0:
            scan -= 1
        log(f"  Previous symbol string starts at 0x{scan:x}: {kc[scan:scan+50]}")
        
        # Scan forward for more symbols
        scan2 = idx + len(target_sym)
        for i in range(10):
            end = kc.find(b'\x00', scan2)
            if end == -1:
                break
            sym = kc[scan2:end]
            if len(sym) > 0 and len(sym) < 200:
                log(f"  Next symbol [{i}]: {sym.decode('utf-8', errors='replace')}")
            scan2 = end + 1
    
    # ═══════════════════════════════════════════════════════════════
    # 3. Find ml_io_map via LC_SYMTAB parsing
    # ═══════════════════════════════════════════════════════════════
    log("\n" + "=" * 70)
    log(" 3. Parse LC_SYMTAB for ml_io_map address")
    log("=" * 70)
    
    # Parse Mach-O header
    magic = struct.unpack('<I', kc[0:4])[0]
    log(f"  Magic: 0x{magic:08x}")
    
    if magic == 0xFEEDFACF:  # MH_MAGIC_64
        ncmds = struct.unpack('<I', kc[16:20])[0]
        log(f"  Number of load commands: {ncmds}")
        
        offset = 32  # size of mach_header_64
        symtab_off = None
        symtab_nsyms = None
        stroff = None
        strsize = None
        
        for i in range(ncmds):
            if offset + 8 > len(kc):
                break
            cmd, cmdsize = struct.unpack('<II', kc[offset:offset+8])
            
            if cmd == 2:  # LC_SYMTAB
                symoff, nsyms, stroff_val, strsize_val = struct.unpack('<IIII', kc[offset+8:offset+24])
                symtab_off = symoff
                symtab_nsyms = nsyms
                stroff = stroff_val
                strsize = strsize_val
                log(f"  LC_SYMTAB found!")
                log(f"    symoff: 0x{symoff:x}")
                log(f"    nsyms: {nsyms}")
                log(f"    stroff: 0x{stroff_val:x}")
                log(f"    strsize: 0x{strsize_val:x}")
                break
            
            offset += cmdsize
        
        if stroff is not None and symtab_off is not None:
            # Find the string in the string table
            str_start = stroff
            # Search for "_ml_io_map\0" relative to stroff
            target = b"_ml_io_map\x00"
            rel_idx = kc[stroff:stroff+strsize].find(target)
            
            if rel_idx != -1:
                log(f"  Found in string table at stroff+0x{rel_idx:x}")
                
                # Now search nlist entries for this strx
                for sym_i in range(symtab_nsyms):
                    nlist_off = symtab_off + sym_i * 16
                    if nlist_off + 16 > len(kc):
                        break
                    n_strx, n_type, n_sect, n_desc, n_value = struct.unpack('<IBBHI', kc[nlist_off:nlist_off+12])
                    n_value = struct.unpack('<Q', kc[nlist_off+8:nlist_off+16])[0]
                    
                    if n_strx == rel_idx:
                        log(f"  FOUND ml_io_map nlist entry!")
                        log(f"    n_strx:  0x{n_strx:x}")
                        log(f"    n_type:  0x{n_type:02x}")
                        log(f"    n_sect:  {n_sect}")
                        log(f"    n_desc:  0x{n_desc:04x}")
                        log(f"    n_value: 0x{n_value:016x}")
                        
                        if n_value > KC_BASE:
                            log(f"\n  --> ml_io_map @ 0x{n_value:016x}")
                            for line in disasm_func(kc, cs, n_value, max_bytes=4096, label="ml_io_map"):
                                log(line)
                        break
                
                # Also find other interesting symbols nearby
                interesting = [
                    b"_ml_io_unmap\x00",
                    b"_ml_io_map_wcomb\x00",
                    b"_IOMapPages\x00",
                    b"_pmap_enter_addr\x00",
                    b"_pmap_map_bd\x00",
                    b"_gPhysBase\x00",
                    b"_gVirtBase\x00",
                    b"_physmap_base\x00",
                    b"_gPhysSize\x00",
                    b"_ml_static_mfree\x00",
                    b"_kvtophys\x00",
                    b"_phystokv\x00",
                    b"_bcopy_phys\x00",
                    b"_pmap_find_phys\x00",
                ]
                
                log(f"\n--- Other kernel symbols ---")
                for sym_name in interesting:
                    rel = kc[stroff:stroff+strsize].find(sym_name)
                    if rel == -1:
                        log(f"  {sym_name.decode().strip(chr(0)):30s}  NOT in symtab")
                        continue
                    
                    # Find matching nlist
                    found_val = None
                    for sym_i in range(symtab_nsyms):
                        nlist_off = symtab_off + sym_i * 16
                        if nlist_off + 16 > len(kc):
                            break
                        n_strx = struct.unpack('<I', kc[nlist_off:nlist_off+4])[0]
                        if n_strx == rel:
                            n_value = struct.unpack('<Q', kc[nlist_off+8:nlist_off+16])[0]
                            found_val = n_value
                            break
                    
                    if found_val:
                        log(f"  {sym_name.decode().strip(chr(0)):30s}  0x{found_val:016x}")
                    else:
                        log(f"  {sym_name.decode().strip(chr(0)):30s}  strx=0x{rel:x} but no nlist match")
            else:
                log(f"  '_ml_io_map' NOT found in string table region")
    
    # ═══════════════════════════════════════════════════════════════
    # 4. Disassemble kvtophys function
    # ═══════════════════════════════════════════════════════════════
    log("\n" + "=" * 70)
    log(" 4. kvtophys @ 0xfffffff008066510")
    log("=" * 70)
    
    KVTOPHYS = 0xfffffff008066510
    for line in disasm_func(kc, cs, KVTOPHYS, label="kvtophys"):
        log(line)
    
    # ═══════════════════════════════════════════════════════════════
    # 5. Check dram-base references  
    # ═══════════════════════════════════════════════════════════════
    log("\n" + "=" * 70)
    log(" 5. dram-base device tree lookup")
    log("=" * 70)
    
    # "dram-base" string at 0x70580fb with surrounding context
    # The init code reads this from device tree to set gPhysBase
    dram_str = kc.find(b"dram-base\x00")
    if dram_str != -1:
        # Look at the surrounding code context
        region = kc[max(0,dram_str-256):dram_str+256]
        log(f"  'dram-base' found at offset 0x{dram_str:x}")
        log(f"  Context: {region[:128]}")
        log(f"  Full string area: {region[200:300]}")
        
        # The code that reads dram-base sets gPhysBase
        # For A13: dram-base = 0x800000000 (typical)
        # This means gPhysBase will be 0x800000000 at runtime
        # BootROM at 0x100000000 is BELOW this -> ml_phys_read will panic
        log(f"\n  A13 (T8030) typical memory layout:")
        log(f"    SecureROM:     0x100000000 - 0x100080000  (512 KB)")
        log(f"    SRAM:          0x190000000 - ...           (varies)")
        log(f"    DRAM base:     0x800000000                (boot-arg)")
        log(f"    DRAM end:      depends on model (4/6 GB)")
        log(f"    gPhysBase:     = dram-base = 0x800000000")
        log(f"    gPhysSize:     = DRAM size  ~ 0x100000000 (4GB)")
        log(f"    ")
        log(f"    BootROM (0x100000000) < gPhysBase (0x800000000)")
        log(f"    --> ml_phys_read WILL PANIC for BootROM address")
        log(f"    --> Must use ml_io_map or patch gPhysBase")
    
    # Save
    log("\n" + "=" * 70)
    log(" SUMMARY")
    log("=" * 70)
    
    with open(OUTPUT, 'w', encoding='utf-8') as f:
        f.write('\n'.join(out))
    print(f"\n[+] Saved to {OUTPUT} ({len(out)} lines)")


if __name__ == '__main__':
    main()
