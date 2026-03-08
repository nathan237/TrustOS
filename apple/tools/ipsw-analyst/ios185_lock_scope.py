#!/usr/bin/env python3
"""Quick function boundary analysis — find retab/ret and BL targets in each function."""
import struct
from pathlib import Path
from capstone import *

KC_PATH = Path("extracted/kernelcache_iPhone12,3_18_5.raw")
KC_BASE = 0xfffffff007004000

LOCK_FUNC = 0xfffffff007f4bc20
UNLOCK_FUNC = 0xfffffff007f4c894

FUNCTIONS = {
    "s_set_value":       0xfffffff00857ee7c,
    "s_get_value":       0xfffffff00857f3a4,
    "s_set_value_xml":   0xfffffff00857e1ac,
    "s_get_value_xml":   0xfffffff00857dde8,
    "s_decrement_use":   0xfffffff00857e464,
    "s_increment_use":   0xfffffff00861c8d0,
    "s_create_surface":  0xfffffff009e86024,
    "s_bulk_get_value":  0xfffffff00857d83c,
    "s_bulk_set_value":  0xfffffff00857dba0,
}

def va2off(va):
    return va - KC_BASE

def main():
    kc = Path(KC_PATH).read_bytes()
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = False
    
    print("=" * 80)
    print(" FUNCTION BOUNDARY + LOCK SCOPE ANALYSIS")
    print("=" * 80)
    
    for name, va in sorted(FUNCTIONS.items(), key=lambda x: x[1]):
        off = va2off(va)
        chunk = kc[off:off + 4000]  # Up to ~1000 instructions
        instrs = list(md.disasm(chunk, va, count=1000))
        
        # Find ALL retab/ret instructions — function may have multiple returns
        rets = []
        for i, insn in enumerate(instrs):
            if insn.mnemonic in ('ret', 'retab'):
                rets.append((i, insn.address))
        
        # Find ALL pacibsp (function prologs) — to detect embedded sub-functions
        prologs = []
        for i, insn in enumerate(instrs):
            if insn.mnemonic == 'pacibsp' and i > 0:  # skip first (it's our function)
                prologs.append((i, insn.address))
        
        # Determine TRUE function end:
        # It's either the first retab, OR if there's a pacibsp right after, 
        # the function boundary is at the pacibsp
        first_ret_idx = rets[0][0] if rets else len(instrs)
        first_ret_va = rets[0][1] if rets else va + len(instrs)*4
        
        # Check if there's a pacibsp right after the first ret
        # (indicating a new function starts)
        true_end = first_ret_va
        for pi, pva in prologs:
            if pva < true_end:
                true_end = pva - 4
                break
        
        func_size = true_end - va + 4
        
        # Find lock/unlock within function boundary
        lock_calls = []
        unlock_calls = []
        bl_targets = []
        vtable_dispatches = []
        inner_loads = []
        
        for insn in instrs:
            if insn.address > true_end:
                break
            
            if insn.mnemonic == 'bl':
                try:
                    target = int(insn.op_str.lstrip('#'), 16)
                    bl_targets.append((insn.address, target))
                    if target == LOCK_FUNC:
                        lock_calls.append(insn.address)
                    elif target == UNLOCK_FUNC:
                        unlock_calls.append(insn.address)
                except:
                    pass
            
            if insn.mnemonic in ('blraa', 'braa'):
                vtable_dispatches.append(insn.address)
            
            # LDR from self+0x18 (inner object)
            if insn.mnemonic == 'ldr' and '#0x18]' in insn.op_str:
                inner_loads.append(insn.address)
        
        # Determine lock scope
        has_entry_lock = any(l < va + 0x40 for l in lock_calls)  # Lock near entry
        has_exit_unlock = any(u > true_end - 0x40 for u in unlock_calls)
        
        print(f"\n{'─'*80}")
        print(f"  {name} @ 0x{va:x}")
        print(f"  Size: {func_size} bytes  |  First ret/retab: 0x{first_ret_va:x}")
        print(f"  Next prolog (pacibsp): {f'0x{prologs[0][1]:x}' if prologs else 'N/A'}")
        print(f"  BL lock acquire (0x7f4bc20): {[f'0x{l:x}' for l in lock_calls] or 'NONE'}")
        print(f"  BL lock release (0x7f4c894): {[f'0x{u:x}' for u in unlock_calls] or 'NONE'}")
        print(f"  Vtable dispatches (BLRAA): {len(vtable_dispatches)}")
        print(f"  Inner object loads (self+0x18): {[f'0x{a:x}' for a in inner_loads]}")
        
        if lock_calls:
            first_lock = min(lock_calls)
            last_unlock = max(unlock_calls) if unlock_calls else 0
            
            # Check for dispatches BEFORE lock
            pre_lock_dispatches = [d for d in vtable_dispatches if d < first_lock]
            post_unlock_dispatches = [d for d in vtable_dispatches if d > last_unlock] if last_unlock else []
            
            print(f"  Lock scope: 0x{first_lock:x} - 0x{last_unlock:x}" if last_unlock else f"  Lock at: 0x{first_lock:x}")
            print(f"  Dispatches BEFORE lock: {len(pre_lock_dispatches)} {[f'0x{d:x}' for d in pre_lock_dispatches]}")
            print(f"  Dispatches AFTER unlock: {len(post_unlock_dispatches)} {[f'0x{d:x}' for d in post_unlock_dispatches]}")
            
            # Inner object loads before lock?
            pre_lock_inner = [a for a in inner_loads if a < first_lock]
            if pre_lock_inner:
                print(f"  ⚠️  INNER OBJECT LOAD BEFORE LOCK: {[f'0x{a:x}' for a in pre_lock_inner]}")
        else:
            dispatches_in_func = vtable_dispatches
            if dispatches_in_func:
                print(f"  ⚠️  NO LOCK — {len(dispatches_in_func)} vtable dispatches UNPROTECTED")
            else:
                print(f"  (No vtable dispatches in this function)")
        
        # Print ADRP targets for lock argument
        for insn in instrs:
            if insn.address > true_end:
                break
            if insn.mnemonic == 'adrp' and insn.address < (lock_calls[0] - 4 if lock_calls else va + 0x40):
                # Check if this is setting up the lock argument
                try:
                    parts = insn.op_str.split(',')
                    if parts[0].strip() == 'x0':
                        base = int(parts[1].strip().lstrip('#'), 16)
                        page = (insn.address & ~0xFFF) + base
                        # Check next instruction for ADD
                        next_off = va2off(insn.address + 4)
                        next_bytes = kc[next_off:next_off+4]
                        next_insn = list(md.disasm(next_bytes, insn.address + 4, count=1))
                        if next_insn and next_insn[0].mnemonic == 'add':
                            add_parts = next_insn[0].op_str.split(',')
                            add_imm = int(add_parts[-1].strip().lstrip('#'), 16)
                            lock_arg = page + add_imm
                            print(f"  Lock argument: 0x{lock_arg:x}")
                except:
                    pass
        
        # Print first 5 instructions context
        print(f"  First instructions:")
        for insn in instrs[:8]:
            marker = ""
            if insn.mnemonic == 'bl':
                try:
                    t = int(insn.op_str.lstrip('#'), 16)
                    if t == LOCK_FUNC:
                        marker = " ◄ LOCK"
                    elif t == UNLOCK_FUNC:
                        marker = " ◄ UNLOCK"
                except:
                    pass
            if insn.mnemonic == 'ldr' and '#0x18]' in insn.op_str:
                marker = " ◄ INNER OBJ"
            print(f"    0x{insn.address:x}: {insn.mnemonic:8s} {insn.op_str}{marker}")
    
    # Critical analysis: same lock or different locks?
    print(f"\n{'='*80}")
    print(" LOCK FUNCTION ANALYSIS — Is it the same lock?")
    print(f"{'='*80}")
    
    # Disassemble lock_func to find what global it accesses
    off = va2off(LOCK_FUNC)
    chunk = kc[off:off + 200]
    instrs = list(md.disasm(chunk, LOCK_FUNC, count=50))
    print(f"  Lock function @ 0x{LOCK_FUNC:x} first 15 instructions:")
    for insn in instrs[:15]:
        print(f"    0x{insn.address:x}: {insn.mnemonic:8s} {insn.op_str}")
    
    # The lock function receives its argument in x0 (passed by each caller)
    # So if callers pass different x0 values, they use different locks!
    print(f"\n  KEY INSIGHT: Lock function receives lock object in x0")
    print(f"  Each caller passes the lock via ADRP+ADD before BL")
    print(f"  If all callers pass the SAME address → same lock → no race")
    print(f"  If callers pass DIFFERENT addresses → different locks → RACE POSSIBLE")

if __name__ == "__main__":
    main()
