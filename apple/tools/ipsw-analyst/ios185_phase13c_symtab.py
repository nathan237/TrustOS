#!/usr/bin/env python3
"""
Phase 13c v2: Fix LC_FILESET_ENTRY constant (0x80000035), parse full MH_FILESET,
resolve ALL critical kernel symbols via nlist64 tables.
"""

import struct
from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM

KC_PATH = "extracted/kernelcache_iPhone12,3_18_5.raw"
KC_BASE = 0xfffffff007004000
OUTPUT = "phase13c_out.txt"

LC_SYMTAB          = 0x02
LC_SEGMENT_64      = 0x19
LC_UUID            = 0x1B
LC_SOURCE_VERSION  = 0x2A
LC_BUILD_VERSION   = 0x32
LC_FILESET_ENTRY   = 0x80000035  # LC_REQ_DYLD | 0x35

def va_to_off(va):
    return va - KC_BASE

def off_to_va(off):
    return off + KC_BASE

def parse_lc(kc, header_off, ncmds):
    lcs = []
    off = header_off + 32
    for i in range(ncmds):
        if off + 8 > len(kc):
            break
        cmd, cmdsize = struct.unpack('<II', kc[off:off+8])
        lcs.append((cmd, off, cmdsize))
        off += cmdsize
    return lcs

def main():
    with open(KC_PATH, 'rb') as f:
        kc = f.read()
    
    out = []
    def log(msg=""):
        print(msg)
        out.append(msg)
    
    cs = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    cs.detail = True
    
    log("=" * 70)
    log("PHASE 13C: MH_FILESET Symbol Resolution")
    log("=" * 70)
    
    ncmds = struct.unpack('<I', kc[16:20])[0]
    log(f"  ncmds={ncmds}")
    
    lcs = parse_lc(kc, 0, ncmds)
    
    cmd_counts = {}
    for cmd, off, sz in lcs:
        cmd_counts[cmd] = cmd_counts.get(cmd, 0) + 1
    
    log(f"\n  Outer load command types:")
    for cmd, count in sorted(cmd_counts.items()):
        log(f"    cmd=0x{cmd:08x} count={count}")
    
    # Gather fileset entries + outer segments + outer symtab
    fileset_entries = []
    outer_segments = {}
    outer_symtab = None
    
    for cmd, off, sz in lcs:
        if cmd == LC_FILESET_ENTRY:
            vmaddr = struct.unpack('<Q', kc[off+8:off+16])[0]
            fileoff = struct.unpack('<Q', kc[off+16:off+24])[0]
            entry_id_off = struct.unpack('<I', kc[off+24:off+28])[0]
            str_start = off + entry_id_off
            str_end = kc.find(b'\x00', str_start)
            if str_end == -1:
                str_end = str_start + 256
            name = kc[str_start:str_end].decode('utf-8', errors='replace')
            fileset_entries.append((name, vmaddr, fileoff))
        elif cmd == LC_SEGMENT_64:
            segname = kc[off+8:off+24].split(b'\x00')[0].decode()
            vmaddr = struct.unpack('<Q', kc[off+24:off+32])[0]
            vmsize = struct.unpack('<Q', kc[off+32:off+40])[0]
            fileoff = struct.unpack('<Q', kc[off+40:off+48])[0]
            filesize = struct.unpack('<Q', kc[off+48:off+56])[0]
            outer_segments[segname] = {'vmaddr': vmaddr, 'vmsize': vmsize, 'fileoff': fileoff, 'filesize': filesize}
        elif cmd == LC_SYMTAB:
            symoff, nsyms, stroff, strsize = struct.unpack('<IIII', kc[off+8:off+24])
            outer_symtab = (symoff, nsyms, stroff, strsize)
    
    log(f"\n  Fileset entries: {len(fileset_entries)}")
    log(f"  Outer segments: {list(outer_segments.keys())}")
    if outer_symtab:
        log(f"  Outer LC_SYMTAB: symoff=0x{outer_symtab[0]:x} nsyms={outer_symtab[1]} stroff=0x{outer_symtab[2]:x} strsize=0x{outer_symtab[3]:x}")
    else:
        log(f"  Outer LC_SYMTAB: None")
    
    # Log key entries
    for name, vmaddr, fileoff in fileset_entries:
        if any(k in name for k in ['kernel', 'IOSurface', 'AppleARM']):
            log(f"    '{name}' vmaddr=0x{vmaddr:016x} fileoff=0x{fileoff:x}")
    
    # ═══════════════════════════════════════════════════════════════
    # Parse each fileset sub-entry for LC_SYMTAB
    # ═══════════════════════════════════════════════════════════════
    log(f"\n{'=' * 70}")
    log(f"Scanning sub-entries for LC_SYMTAB")
    log(f"{'=' * 70}")
    
    all_symtabs = []
    if outer_symtab:
        all_symtabs.append(('OUTER', *outer_symtab))
    
    for name, vmaddr, fileoff in fileset_entries:
        if fileoff + 32 > len(kc):
            continue
        m = struct.unpack('<I', kc[fileoff:fileoff+4])[0]
        if m != 0xFEEDFACF:
            continue
        nc = struct.unpack('<I', kc[fileoff+16:fileoff+20])[0]
        sub_lcs = parse_lc(kc, fileoff, nc)
        for cmd, off, sz in sub_lcs:
            if cmd == LC_SYMTAB:
                symoff, nsyms, stroff, strsize = struct.unpack('<IIII', kc[off+8:off+24])
                if nsyms > 0:
                    log(f"  [{name[:50]:50s}] nsyms={nsyms:6d} symoff=0x{symoff:08x} stroff=0x{stroff:08x}")
                    all_symtabs.append((name, symoff, nsyms, stroff, strsize))
    
    log(f"\n  Total symbol tables found: {len(all_symtabs)}")
    
    # ═══════════════════════════════════════════════════════════════
    # Resolve symbols from all tables
    # ═══════════════════════════════════════════════════════════════
    target_symbols = [
        "_ml_io_map", "_ml_io_unmap", "_ml_io_map_wcomb", "_ml_io_map_in_range",
        "_ml_phys_read_data", "_ml_phys_write_data", "_ml_phys_read_double_data",
        "_ml_phys_read_byte_64", "_ml_phys_write_double_data",
        "_gPhysBase", "_gPhysSize", "_gVirtBase",
        "_physmap_base", "_physmap_end",
        "_kvtophys", "_phystokv",
        "_bcopy_phys", "_pmap_find_phys", "_pmap_enter", "_pmap_remove",
        "_pmap_map_bd", "_pmap_map_io_range",
        "_IOMapPages", "_ml_static_mfree",
        "_copyin", "_copyout",
        "_kernel_task", "_kernel_map", "_current_task", "_allproc",
        "_IOLockLock", "_IOLockUnlock",
    ]
    
    resolved = {}
    total_scanned = 0
    
    for tab_name, symoff, nsyms, stroff, strsize in all_symtabs:
        if symoff == 0 or stroff == 0 or nsyms == 0:
            continue
        safe_end = symoff + nsyms * 16
        if safe_end > len(kc) or stroff + strsize > len(kc):
            log(f"  SKIP {tab_name}: out of bounds (symoff=0x{symoff:x} nsyms={nsyms} safe_end=0x{safe_end:x} kc_len=0x{len(kc):x})")
            continue
        
        strtab = kc[stroff:stroff+strsize]
        total_scanned += nsyms
        
        for si in range(nsyms):
            noff = symoff + si * 16
            n_strx = struct.unpack('<I', kc[noff:noff+4])[0]
            n_type = kc[noff+4]
            n_sect = kc[noff+5]
            n_desc = struct.unpack('<H', kc[noff+6:noff+8])[0]
            n_value = struct.unpack('<Q', kc[noff+8:noff+16])[0]
            
            if n_strx >= strsize or n_value == 0:
                continue
            
            end = strtab.find(b'\x00', n_strx)
            if end == -1:
                continue
            sname = strtab[n_strx:end].decode('utf-8', errors='replace')
            
            if sname in target_symbols and sname not in resolved:
                resolved[sname] = (n_value, tab_name, n_type, n_sect)
    
    log(f"\n  Scanned {total_scanned} nlist entries from {len(all_symtabs)} tables")
    log(f"  Resolved: {len(resolved)} / {len(target_symbols)}")
    
    # ═══════════════════════════════════════════════════════════════
    # If no symtabs found, fallback: direct __LINKEDIT pattern scan
    # ═══════════════════════════════════════════════════════════════
    if len(resolved) == 0 and '__LINKEDIT' in outer_segments:
        log(f"\n{'=' * 70}")
        log(f"FALLBACK: Direct __LINKEDIT nlist64 pattern scan")
        log(f"{'=' * 70}")
        
        le = outer_segments['__LINKEDIT']
        le_off = le['fileoff']
        le_size = le['filesize']
        le_end = le_off + le_size
        
        log(f"  __LINKEDIT: file 0x{le_off:x}..0x{le_end:x} ({le_size} bytes, {le_size/(1024*1024):.1f} MB)")
        
        # Find known symbol strings in __LINKEDIT
        known_str_targets = {}
        for sym in target_symbols:
            pattern = sym.encode() + b'\x00'
            pos = kc.find(pattern, le_off, le_end)
            if pos != -1:
                known_str_targets[sym] = pos
                log(f"    String '{sym}' at file 0x{pos:x}")
        
        if not known_str_targets:
            log(f"  No target strings found in __LINKEDIT!")
        else:
            # Use a known function address to calibrate nlist/stroff
            # We know phystokv = 0xfffffff0080772c8
            known_funcs = {
                0xfffffff0080772c8: '_phystokv',
                0xfffffff008066510: '_kvtophys',
                0xfffffff00807b4f8: '_ml_phys_read_core',  # internal name might differ
                0xfffffff00807b738: '_ml_phys_read_data',
            }
            
            for known_va, expected_name in known_funcs.items():
                if expected_name not in known_str_targets:
                    continue
                
                va_bytes = struct.pack('<Q', known_va)
                str_file_pos = known_str_targets[expected_name]
                
                # Find this VA in __LINKEDIT (at offset +8 in an nlist64 entry)
                pos = le_off
                while pos < le_end - 16:
                    pos = kc.find(va_bytes, pos, le_end)
                    if pos == -1:
                        break
                    
                    # Check if this is at nlist64 n_value position (offset +8)
                    nlist_start = pos - 8
                    if nlist_start < le_off:
                        pos += 1
                        continue
                    
                    n_strx = struct.unpack('<I', kc[nlist_start:nlist_start+4])[0]
                    n_type = kc[nlist_start+4]
                    
                    # Compute what stroff would be
                    computed_stroff = str_file_pos - n_strx
                    
                    # Validate by checking the string
                    if computed_stroff >= 0 and computed_stroff + n_strx < len(kc):
                        check_end = kc.find(b'\x00', computed_stroff + n_strx, computed_stroff + n_strx + 256)
                        if check_end != -1:
                            check_str = kc[computed_stroff + n_strx:check_end].decode('utf-8', errors='replace')
                            if check_str == expected_name:
                                log(f"\n  CALIBRATED via {expected_name}:")
                                log(f"    nlist @ file 0x{nlist_start:x}")
                                log(f"    n_strx=0x{n_strx:x} n_type=0x{n_type:02x}")
                                log(f"    stroff = 0x{computed_stroff:x}")
                                
                                # Determine symoff by scanning backwards
                                scan = nlist_start - 16
                                while scan >= le_off:
                                    sv = struct.unpack('<Q', kc[scan+8:scan+16])[0]
                                    if (sv >> 28) != 0xffffffff0:
                                        # might still be valid (globals can have different patterns)
                                        # But kernel VAs should start with 0xfffffff0
                                        st = kc[scan+4]
                                        if st == 0 and sv == 0:
                                            scan -= 16
                                            continue
                                        scan += 16
                                        break
                                    scan -= 16
                                
                                symoff = max(scan, le_off)
                                
                                # Scan forward to find end
                                scan_f = nlist_start
                                while scan_f < le_end - 16:
                                    sv = struct.unpack('<Q', kc[scan_f+8:scan_f+16])[0]
                                    st = kc[scan_f+4]
                                    if sv == 0 and st == 0:
                                        break
                                    if (sv >> 28) != 0xffffffff0 and sv != 0:
                                        break
                                    scan_f += 16
                                
                                nsyms = (scan_f - symoff) // 16
                                
                                log(f"    symoff ≈ 0x{symoff:x}")
                                log(f"    nsyms ≈ {nsyms}")
                                log(f"    Resolving all target symbols...")
                                
                                # Now scan all nlist entries in this range
                                for si in range(nsyms):
                                    noff = symoff + si * 16
                                    ns = struct.unpack('<I', kc[noff:noff+4])[0]
                                    nt = kc[noff+4]
                                    nsec = kc[noff+5]
                                    nv = struct.unpack('<Q', kc[noff+8:noff+16])[0]
                                    
                                    if nv == 0:
                                        continue
                                    
                                    str_pos = computed_stroff + ns
                                    if str_pos < 0 or str_pos >= len(kc):
                                        continue
                                    end = kc.find(b'\x00', str_pos, str_pos + 256)
                                    if end == -1:
                                        continue
                                    sname = kc[str_pos:end].decode('utf-8', errors='replace')
                                    
                                    if sname in target_symbols and sname not in resolved:
                                        resolved[sname] = (nv, 'LINKEDIT_calibrated', nt, nsec)
                                
                                log(f"    Resolved {len(resolved)} symbols")
                                break
                    pos += 1
                
                if len(resolved) > 0:
                    break
    
    # ═══════════════════════════════════════════════════════════════
    # Print final results
    # ═══════════════════════════════════════════════════════════════
    log(f"\n{'=' * 70}")
    log(f"FINAL SYMBOL RESOLUTION ({len(resolved)}/{len(target_symbols)})")
    log(f"{'=' * 70}\n")
    
    for sym_name in target_symbols:
        if sym_name in resolved:
            val, tab, ntype, nsect = resolved[sym_name]
            off = val - KC_BASE
            log(f"  {sym_name:35s} = 0x{val:016x}  (file +0x{off:x})")
        else:
            log(f"  {sym_name:35s}   ---")
    
    # Cross-validate
    log(f"\n{'=' * 70}")
    log(f"Cross-validation with disasm-derived addresses")
    log(f"{'=' * 70}")
    
    known = {
        "phystokv":          0xfffffff0080772c8,
        "kvtophys":          0xfffffff008066510,
        "ml_phys_read_core": 0xfffffff00807b4f8,
        "ml_phys_read_data": 0xfffffff00807b738,
        "gPhysBase (data)":  0xfffffff007a6cbb8,
        "gPhysSize (data)":  0xfffffff007a6cbc0,
    }
    
    for label, addr in known.items():
        match = any(v[0] == addr for v in resolved.values())
        sym_key = "_" + label.split(" ")[0]
        sym_val = resolved.get(sym_key, (None,))[0]
        if sym_val == addr:
            log(f"  {label:30s} 0x{addr:016x}  MATCH")
        elif sym_val:
            log(f"  {label:30s} 0x{addr:016x}  symtab=0x{sym_val:016x} MISMATCH!")
        else:
            log(f"  {label:30s} 0x{addr:016x}  (from disasm, not in symtab)")
    
    # Disassemble ml_io_map if found
    for fn_name in ["_ml_io_map", "_ml_io_unmap", "_ml_io_map_wcomb"]:
        if fn_name not in resolved:
            continue
        fn_va = resolved[fn_name][0]
        log(f"\n{'=' * 70}")
        log(f"DISASSEMBLY: {fn_name} @ 0x{fn_va:016x}")
        log(f"{'=' * 70}")
        
        off = va_to_off(fn_va)
        if off < 0 or off >= len(kc) - 4096:
            log(f"  ERROR: offset 0x{off:x} out of range")
            continue
        
        chunk = kc[off:off+4096]
        ic = 0
        bls = []
        for insn in cs.disasm(chunk, fn_va):
            log(f"  0x{insn.address:x}: {insn.mnemonic:10s} {insn.op_str}")
            ic += 1
            if insn.mnemonic == 'bl':
                try:
                    bls.append(int(insn.op_str.replace('#', ''), 16))
                except:
                    pass
            if insn.mnemonic in ('ret', 'retab', 'retaa') and ic > 3:
                break
            if ic > 150:
                break
        log(f"  [{ic} instructions]")
        if bls:
            log(f"  BL targets: {', '.join(f'0x{b:x}' for b in bls)}")
    
    # ═══════════════════════════════════════════════════════════════
    # BootROM Strategy
    # ═══════════════════════════════════════════════════════════════
    gPB = resolved.get("_gPhysBase", (0xfffffff007a6cbb8,))[0]
    gPS = resolved.get("_gPhysSize", (0xfffffff007a6cbc0,))[0]
    ml_io = resolved.get("_ml_io_map", (None,))[0]
    
    log(f"\n{'=' * 70}")
    log(f"BOOTROM ACCESS STRATEGY (A13/T8030, iOS 18.5)")
    log(f"{'=' * 70}")
    log(f"""
  Target: SecureROM 0x100000000..0x100080000 (512 KB)
  
  PROBLEM: ml_phys_read checks paddr in [gPhysBase, gPhysBase+gPhysSize)
           gPhysBase = 0x800000000 (DRAM), BootROM = 0x100000000 < gPhysBase
  
  === APPROACH A: gPhysBase/gPhysSize Patching [PREFERRED] ===
  Requires: kernel R/W primitive (from IOSurface race)
  
    gPhysBase @ 0x{gPB:016x} + KASLR slide
    gPhysSize @ 0x{gPS:016x} + KASLR slide
    
    1. orig_base = kread64(gPhysBase + slide)
       orig_size = kread64(gPhysSize + slide)
    2. kwrite64(gPhysBase + slide, 0x0)
       kwrite64(gPhysSize + slide, 0x200000000)
    3. for i in range(0, 0x80000, 8):
         buf[i:i+8] = call_ml_phys_read_double(0x100000000 + i)
    4. kwrite64(gPhysBase + slide, orig_base)
       kwrite64(gPhysSize + slide, orig_size)
    
    Note: Step 3 needs kernel execute. With kernel R/W only:
    - Read via physmap: physmap_va = physmap_base + (phys - gPhysBase)
    - After patching gPhysBase=0: physmap_va = 0xfffffffbffd00000 + 0x100000000
      = 0xfffffffc0fd00000 (check if this maps correctly with new gPhysBase)
    
    Alternative for step 3 (R/W only, no exec):
    - After patching gPhysBase to 0, the physmap offset changes
    - physmap covers gPhysBase to gPhysBase+gPhysSize
    - New range: 0x0 to 0x200000000
    - BootROM physmap VA = physmap_base + 0x100000000
      = 0xfffffffbffd00000 + 0x100000000 = 0xfffffffc0fd00000
    - BUT: physmap page tables may not cover this new range!
    - physmap was set up at boot for the ORIGINAL gPhysBase range
    - Page tables only map DRAM range, not BootROM range
    - => Patching globals alone is NOT ENOUGH for physmap reads
    - => Need ml_phys_read (which creates temporary mapping) or ml_io_map
""")
    
    if ml_io:
        log(f"""  === APPROACH B: ml_io_map [SAFEST] ===
  Requires: kernel execute primitive
  
    ml_io_map @ 0x{ml_io:016x} + KASLR slide
    
    1. mapped_va = kexec(ml_io_map + slide, 0x100000000, 0x80000)
    2. for i in range(0, 0x80000, 8):
         buf[i:i+8] = kread64(mapped_va + i)
    3. kexec(ml_io_unmap + slide, mapped_va, 0x80000)
    
    Advantage: No global patching, no race window
""")
    
    log(f"""  === APPROACH C: Direct ml_phys_read via gPhysBase patch ===
  The DEFINITIVE approach combining A's simplicity with correctness:
  
    ml_phys_read internally does:
      1. Check paddr in [gPhysBase, gPhysBase+gPhysSize) — we patch this
      2. Temporarily maps the physical page via physmap or dedicated mapping
      3. Reads the value and returns it
    
    ml_phys_read does NOT use physmap for the actual read!
    It uses a DEDICATED temporary mapping mechanism.
    
    Therefore: patching gPhysBase=0 + gPhysSize=0x200000000 is SUFFICIENT
    for ml_phys_read to work on BootROM addresses.
    
    The only question is whether ml_phys_read_core's actual read path
    goes through physmap_base or through a separate pagetable entry.
    This needs verification from the ml_phys_read_core disassembly.
""")
    
    # Verify: check ml_phys_read_core's actual read mechanism
    log(f"\n{'=' * 70}")
    log(f"VERIFY: ml_phys_read_core read mechanism")
    log(f"{'=' * 70}")
    
    core_va = 0xfffffff00807b4f8
    core_off = va_to_off(core_va)
    chunk = kc[core_off:core_off+512]
    
    log(f"  ml_phys_read_core @ 0x{core_va:x}:")
    ic = 0
    for insn in cs.disasm(chunk, core_va):
        log(f"  0x{insn.address:x}: {insn.mnemonic:10s} {insn.op_str}")
        ic += 1
        if insn.mnemonic in ('ret', 'retab', 'retaa') and ic > 5:
            break
        if ic > 120:
            break
    log(f"  [{ic} instructions]")
    
    with open(OUTPUT, 'w', encoding='utf-8') as f:
        f.write('\n'.join(out))
    log(f"\n[+] Saved to {OUTPUT} ({len(out)} lines)")


if __name__ == '__main__':
    main()
