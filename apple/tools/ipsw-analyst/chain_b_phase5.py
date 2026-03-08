#!/usr/bin/env python3
"""
Chain B Phase 5 — Multiplication Core + kernel_call Primitive
==============================================================
Traces the EXACT multiplication function at 0xfffffff00a1d0384
(called from IOSurface_max_check) and searches for kernel_call
gadgets that can invoke ml_phys_read from corrupted vtable.
"""

import struct
import json
import sys
from pathlib import Path

try:
    from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM
except ImportError:
    print("[!] capstone required"); sys.exit(1)

EXTRACTED = Path("extracted")
KC_BASE = 0xfffffff007004000

IOSURFACE_TEXT_EXEC_START = 0xfffffff00a1c5c80
IOSURFACE_TEXT_EXEC_END   = 0xfffffff00a1f75dc

def va_to_file(va): return va - KC_BASE
def file_to_va(off): return KC_BASE + off

def disasm_function(md, data, va, max_insns=500):
    foff = va_to_file(va)
    if foff < 0 or foff + 4 > len(data): return []
    code = data[foff:foff + min(max_insns * 4, len(data) - foff)]
    insns = list(md.disasm(code, va))
    func = []
    for i in insns:
        func.append(i)
        if i.mnemonic in ("ret", "retab") and len(func) > 3:
            break
        if len(func) >= max_insns:
            break
    return func

def annotate_va(data, va):
    foff = va_to_file(va)
    if 0 <= foff < len(data) - 4:
        try:
            s = data[foff:foff+80]
            null = s.index(0)
            if null > 3: return f'"{s[:null].decode("ascii", errors="replace")}"'
        except: pass
    return f"0x{va:x}"


def main():
    print("=" * 70)
    print("CHAIN B PHASE 5 — MULTIPLICATION CORE + kernel_call GADGETS")
    print("=" * 70)

    kc_path = None
    for f in EXTRACTED.iterdir():
        if f.name.endswith(".raw") and "kernelcache" in f.name:
            kc_path = f; break
    data = kc_path.read_bytes()
    print(f"[*] Loaded {kc_path} ({len(data)/1024/1024:.1f} MB)")

    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = False

    # ================================================================
    # 1. DISASSEMBLE THE MULTIPLICATION CORE (0xfffffff00a1d0384)
    # ================================================================
    print("\n" + "=" * 70)
    print("1. MULTIPLICATION CORE — 0xfffffff00a1d0384")
    print("   (Called from IOSurface_max_check at 0xfffffff00a1d02d0)")
    print("=" * 70)

    mul_core_va = 0xfffffff00a1d0384
    insns = disasm_function(md, data, mul_core_va, max_insns=200)
    print(f"  Instructions: {len(insns)}")

    muls_found = []
    cmps_found = []
    bl_found = []

    for insn in insns:
        ann = ""
        if insn.mnemonic in ("mul", "madd", "umull", "smull", "umaddl", "umulh", "smulh"):
            ann = "  *** MULTIPLY ***"
            muls_found.append((insn.address, insn.mnemonic, insn.op_str))
        elif insn.mnemonic == "lsl":
            ann = "  shift"
        elif insn.mnemonic in ("cmp", "cbnz", "cbz", "tbnz", "tbz"):
            ann = "  *** CHECK ***"
            cmps_found.append((insn.address, insn.mnemonic, insn.op_str))
        elif insn.mnemonic == "lsr" and "#0x20" in insn.op_str:
            ann = "  *** LSR #32 OVERFLOW CHECK ***"
            cmps_found.append((insn.address, insn.mnemonic, insn.op_str))
        elif insn.mnemonic == "bl":
            try:
                t = int(insn.op_str.lstrip("#"), 16)
                ann = f"  // -> {annotate_va(data, t)}"
                bl_found.append((insn.address, t))
            except: pass
        elif insn.mnemonic.startswith(("ldr", "str")) and "#0x" in insn.op_str:
            known = {0x58: "width", 0x60: "height", 0x78: "bpe", 0x80: "elem_w",
                     0x88: "elem_h", 0x90: "bpr", 0x98: "alloc_sz", 0xa0: "total_sz"}
            try:
                off_str = insn.op_str.split("#")[-1].rstrip("]!").strip()
                off = int(off_str, 16) if off_str.startswith("0x") else int(off_str)
                if off in known:
                    ann = f"  // {known[off]}"
            except: pass

        print(f"    0x{insn.address:x}: {insn.mnemonic:8s} {insn.op_str:40s}{ann}")

    print(f"\n  SUMMARY:")
    print(f"    Multiply ops: {len(muls_found)}")
    for a, m, o in muls_found:
        print(f"      0x{a:x}: {m} {o}")
    print(f"    Checks/branches: {len(cmps_found)}")
    for a, m, o in cmps_found:
        print(f"      0x{a:x}: {m} {o}")
    print(f"    BL calls: {len(bl_found)}")
    for a, t in bl_found:
        print(f"      0x{a:x}: bl 0x{t:x}")

    # If the function calls sub-functions, trace them too
    for call_addr, target in bl_found:
        if IOSURFACE_TEXT_EXEC_START <= target <= IOSURFACE_TEXT_EXEC_END:
            print(f"\n  --- SUB-FUNCTION 0x{target:x} (from 0x{call_addr:x}) ---")
            sub_insns = disasm_function(md, data, target, max_insns=200)
            print(f"  Instructions: {len(sub_insns)}")
            sub_muls = []
            for si in sub_insns:
                sann = ""
                if si.mnemonic in ("mul", "madd", "umull", "umaddl", "umulh"):
                    sann = "  *** MULTIPLY ***"
                    sub_muls.append((si.address, si.mnemonic, si.op_str))
                elif si.mnemonic in ("cmp", "cbnz", "cbz"):
                    sann = "  check"
                elif si.mnemonic == "lsr" and "#0x20" in si.op_str:
                    sann = "  *** LSR #32 ***"
                print(f"      0x{si.address:x}: {si.mnemonic:8s} {si.op_str:40s}{sann}")
            if sub_muls:
                print(f"    MUL OPS: {[(f'0x{a:x}', m, o) for a, m, o in sub_muls]}")

    # ================================================================
    # 2. TRACE ALL FUNCTIONS THAT CALL IOSurface_max_check
    # ================================================================
    print("\n" + "=" * 70)
    print("2. WHO CALLS IOSurface_max_check? (0xfffffff00a1d02d0)")
    print("=" * 70)

    max_check_va = 0xfffffff00a1d02d0
    max_check_file = va_to_file(max_check_va)

    # Scan IOSurface __TEXT_EXEC for BL instructions targeting max_check
    ios_start_file = va_to_file(IOSURFACE_TEXT_EXEC_START)
    ios_end_file = va_to_file(IOSURFACE_TEXT_EXEC_END)
    callers = []

    for off in range(ios_start_file, ios_end_file, 4):
        insn_raw = struct.unpack_from("<I", data, off)[0]
        if (insn_raw & 0xFC000000) != 0x94000000:  # BL encoding
            continue
        imm26 = insn_raw & 0x3FFFFFF
        if imm26 & (1 << 25):
            imm26 -= (1 << 26)
        target = file_to_va(off) + (imm26 << 2)
        if target == max_check_va:
            callers.append(file_to_va(off))

    print(f"  Found {len(callers)} call sites to IOSurface_max_check:")
    for caller in callers:
        # Show context: 3 instructions before the BL
        ctx_off = va_to_file(caller) - 12
        if ctx_off >= 0:
            ctx = list(md.disasm(data[ctx_off:ctx_off+16], caller - 12))
            for ci in ctx:
                mark = " <-- BL max_check" if ci.address == caller else ""
                print(f"    0x{ci.address:x}: {ci.mnemonic:8s} {ci.op_str}{mark}")
            print()

    # ================================================================
    # 3. SEARCH FOR kernel_call GADGETS
    # ================================================================
    print("\n" + "=" * 70)
    print("3. kernel_call GADGET SEARCH")
    print("   Looking for: LDP x0, x1/MOV x0; BL/BR external")
    print("=" * 70)

    # Strategy: Find functions in IOSurface that:
    # 1. Load x0 from a controllable source (stack, struct field)
    # 2. Call an external (non-IOSurface) function
    # This could serve as a call-what-where gadget

    # Search the entire IOSurface __TEXT_EXEC
    ios_code = data[ios_start_file:ios_end_file]
    ios_insns = list(md.disasm(ios_code, IOSURFACE_TEXT_EXEC_START))

    gadgets = []
    for i, insn in enumerate(ios_insns):
        # Pattern: LDR x0, [xN, #offset] ; ... ; BL <external>
        if insn.mnemonic == "ldr" and insn.op_str.startswith("x0,") and "[" in insn.op_str:
            # Look ahead up to 5 instructions for a BL to external
            for j in range(1, min(6, len(ios_insns) - i)):
                next_insn = ios_insns[i + j]
                if next_insn.mnemonic == "bl":
                    try:
                        target = int(next_insn.op_str.lstrip("#"), 16)
                        # External = outside IOSurface kext
                        if not (IOSURFACE_TEXT_EXEC_START <= target <= IOSURFACE_TEXT_EXEC_END):
                            gadgets.append({
                                "ldr_addr": insn.address,
                                "ldr_op": insn.op_str,
                                "bl_addr": next_insn.address,
                                "bl_target": target,
                                "distance": j,
                            })
                    except: pass
                    break  # stop at first BL
                if next_insn.mnemonic in ("ret", "retab", "b"):
                    break

    print(f"  Found {len(gadgets)} LDR x0 + BL external patterns")
    # Show most interesting (short distance, controllable source)
    gadgets.sort(key=lambda g: g["distance"])
    shown = set()
    for g in gadgets[:30]:
        key = (g["ldr_op"], g["bl_target"])
        if key in shown: continue
        shown.add(key)
        print(f"    LDR x0, {g['ldr_op']:30s} @ 0x{g['ldr_addr']:x}")
        print(f"      -> BL 0x{g['bl_target']:x} ({g['distance']} insns later)")

    # Also find BLRAA/BLRAB patterns (PAC indirect calls)
    print(f"\n  PAC indirect calls (BLRAA/BLRAB) in IOSurface:")
    pac_calls = []
    for insn in ios_insns:
        if insn.mnemonic in ("blraa", "blrab"):
            pac_calls.append((insn.address, insn.mnemonic, insn.op_str))
    print(f"  Total: {len(pac_calls)}")
    for a, m, o in pac_calls[:20]:
        print(f"    0x{a:x}: {m} {o}")

    # ================================================================
    # 4. IOConnectTrap ANALYSIS
    # ================================================================
    print("\n" + "=" * 70)
    print("4. IOConnectTrap HANDLER SEARCH")
    print("   Looking for trap handlers that can be redirected")
    print("=" * 70)

    # Search for IOConnectTrap / getTargetAndTrapForIndex patterns
    # in the kernel (outside IOSurface)
    trap_string_patterns = [
        b"getTargetAndTrap",
        b"IOConnectTrap",
        b"IOExternalTrap",
    ]

    for pattern in trap_string_patterns:
        idx = 0
        finds = []
        while True:
            idx = data.find(pattern, idx)
            if idx == -1: break
            finds.append(idx)
            idx += 1
        if finds:
            print(f"  '{pattern.decode()}': {len(finds)} occurrences")
            for f_off in finds[:5]:
                va = file_to_va(f_off)
                ctx = data[f_off:f_off+60]
                try:
                    null = ctx.index(0)
                    s = ctx[:null].decode("ascii", errors="replace")
                except:
                    s = ctx[:30].hex()
                print(f"    0x{va:x}: {s}")

    # ================================================================
    # 5. SEARCH FOR EXISTING BootROM MAPPINGS
    # ================================================================
    print("\n" + "=" * 70)
    print("5. EXISTING BootROM/MMIO MAPPING SEARCH")
    print("=" * 70)

    # Search for 0x100000000 as an immediate value in the kernelcache
    # This could reveal existing code that maps or accesses BootROM
    bootrom_bytes = struct.pack("<Q", 0x100000000)
    idx = 0
    rom_refs = []
    while True:
        idx = data.find(bootrom_bytes, idx)
        if idx == -1: break
        rom_refs.append(idx)
        idx += 8

    print(f"  0x100000000 as 64-bit value: {len(rom_refs)} occurrences")
    for r in rom_refs[:10]:
        va = file_to_va(r)
        print(f"    0x{va:x} (file 0x{r:x})")

    # Also search for MOV/MOVK patterns that construct 0x100000000
    # MOV x?, #0x1, LSL #32 → MOVZ xN, #1, LSL #32 → encoding: 0xD2A00020 (for x0)
    # General: MOVZ xD, #0x1, LSL #32 → 0xD2A00020 | (Rd)
    movz_pattern = 0xD2A00020  # MOVZ x0, #1, LSL #32
    movz_finds = []
    for off in range(0, len(data) - 4, 4):
        insn = struct.unpack_from("<I", data, off)[0]
        # MOVZ xRd, #1, LSL #32: opcode = 1101_0010_101_00000_0000_0000_000_Rd
        # = 0xD2A00000 | Rd
        if (insn & 0xFFFFFFE0) == 0xD2A00020:
            # This is MOVZ xRd, #1, LSL #32
            rd = insn & 0x1F
            movz_finds.append((off, rd))

    print(f"\n  MOVZ xN, #1, LSL #32 (constructs 0x100000000): {len(movz_finds)} occurrences")
    for off, rd in movz_finds[:15]:
        va = file_to_va(off)
        # Show surrounding context
        ctx_start = max(0, off - 8)
        ctx_insns = list(md.disasm(data[ctx_start:ctx_start+32], file_to_va(ctx_start)))
        for ci in ctx_insns:
            mark = " <-- 0x100000000" if ci.address == va else ""
            print(f"    0x{ci.address:x}: {ci.mnemonic:8s} {ci.op_str}{mark}")
        print()

    # ================================================================
    # 6. FINAL STATUS
    # ================================================================
    print("\n" + "=" * 70)
    print("PHASE 5 — SUMMARY")
    print("=" * 70)
    print(f"  Multiplication core (0xfffffff00a1d0384): {len(insns)} insns")
    print(f"    MUL ops: {len(muls_found)}")
    print(f"    Callers of max_check: {len(callers)}")
    print(f"  kernel_call gadgets: {len(gadgets)} LDR x0+BL external")
    print(f"  PAC indirect calls: {len(pac_calls)}")
    print(f"  BootROM value refs: {len(rom_refs)}")
    print(f"  BootROM MOVZ patterns: {len(movz_finds)}")


if __name__ == "__main__":
    main()
