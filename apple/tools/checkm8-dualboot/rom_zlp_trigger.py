"""
TARGETED ANALYSIS:
1. Full function containing DD88 (D3D0 callback site) — what requests trigger it?
2. Direct zlp_send callers at DAD8 and DDFC — what are they?
3. dwc3_callback_and_free (E0A8) — verify NULL callback skips call
4. Full DFU request handler dispatch table
"""
import struct, sys
from capstone import *

ROM_PATH = "securerom/t8020_B1_securerom.bin"
ROM_BASE = 0x100000000
ACTIVE_END = 0x25000

with open(ROM_PATH, "rb") as f:
    rom = f.read()

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

def disasm_range(start_off, end_off):
    results = []
    for i in md.disasm(rom[start_off:end_off], ROM_BASE + start_off):
        results.append(i)
    return results

def disasm_at(offset, count=40):
    results = []
    for i in md.disasm(rom[offset:offset+count*4], ROM_BASE + offset):
        results.append(i)
    return results

# ============================================================
# PART 1: Function containing DD88 — full context
# Look backwards for function prologue
# ============================================================
print("=" * 70)
print("PART 1: Function containing DD88 (D3D0 callback site)")
print("=" * 70)

# Scan backwards from DD88 for stp with pre-index (function prologue)
print("\n--- Looking for function prologue before DD88 ---")
for off in range(0xDD80, 0xD800, -4):
    instrs = disasm_at(off, 1)
    if instrs and instrs[0].mnemonic == 'stp' and 'sp' in instrs[0].op_str and '!' in instrs[0].op_str:
        print(f"  Function prologue at 0x{ROM_BASE + off:X}")
        func_start = off
        break
else:
    func_start = 0xD800
    print(f"  No prologue found, starting from 0x{ROM_BASE + func_start:X}")

# Disassemble the full function
print(f"\n--- Full function from 0x{ROM_BASE + func_start:X} to ~DD90+end ---")
ret_count = 0
for i in disasm_range(func_start, func_start + 0x300):
    ann = ""
    if i.mnemonic == 'stp' and 'sp' in i.op_str and '!' in i.op_str:
        ann = " ; ← FUNC ENTRY"
    if i.mnemonic == 'ret':
        ann = " ; ← RET"
        ret_count += 1
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            names = {
                ROM_BASE + 0xD368: "ep0_transfer_setup",
                ROM_BASE + 0xD334: "zlp_send",
                ROM_BASE + 0xE0D8: "usb_core_do_transfer",
                ROM_BASE + 0xF3B0: "calloc",
                ROM_BASE + 0xF1EC: "malloc",
                ROM_BASE + 0xF468: "free",
                ROM_BASE + 0xF680: "memalign",
                ROM_BASE + 0x10D80: "bzero",
                ROM_BASE + 0xD2D0: "hal_abort_call",
                ROM_BASE + 0xE158: "ep_abort_wrapper",
                ROM_BASE + 0x8978: "panic",
            }
            if t in names:
                ann = f" ; → {names[t]}"
        except: pass
    if i.mnemonic == 'adr' and '#' in i.op_str:
        try:
            target = int(i.op_str.split('#')[1].strip(), 0)
            if target == ROM_BASE + 0xD3D0:
                ann = " ; ★★★ LOADS D3D0 (standard_device_request_cb) ★★★"
        except: pass
    if i.mnemonic == 'cmp' and '#' in i.op_str:
        try:
            val = int(i.op_str.split('#')[1].strip(), 0)
            usb_types = {
                0: "GET_STATUS", 1: "CLEAR_FEATURE", 3: "SET_FEATURE",
                5: "SET_ADDRESS", 6: "GET_DESCRIPTOR", 7: "SET_DESCRIPTOR",
                8: "GET_CONFIGURATION", 9: "SET_CONFIGURATION",
                0x21: "DFU_class_IN", 0xA1: "DFU_class_OUT",
            }
            if val in usb_types:
                ann = f" ; ★ {usb_types[val]}"
            elif val <= 10:
                ann = f" ; check = {val}"
        except: pass
    if 'ldr' in i.mnemonic and '#0x28' in i.op_str:
        ann += " ; [+0x28] io_buffer ptr?"
    print(f"  0x{i.address:X}: {i.mnemonic:6s} {i.op_str}{ann}")
    if ret_count >= 2 and i.address > ROM_BASE + 0xDD88:
        break

# ============================================================
# PART 2: Direct zlp_send caller at DAD8
# ============================================================
print("\n" + "=" * 70)
print("PART 2: Direct zlp_send caller at DAD8")
print("=" * 70)

# Find function prologue
for off in range(0xDAD0, 0xDA00, -4):
    instrs = disasm_at(off, 1)
    if instrs and instrs[0].mnemonic == 'stp' and 'sp' in instrs[0].op_str and '!' in instrs[0].op_str:
        func_start = off
        print(f"  Function prologue at 0x{ROM_BASE + off:X}")
        break
else:
    func_start = 0xDA00
    print(f"  Starting from 0x{ROM_BASE + func_start:X}")

ret_count = 0
for i in disasm_range(func_start, func_start + 0x200):
    ann = ""
    if i.mnemonic == 'ret':
        ann = " ; ← RET"
        ret_count += 1
    if i.mnemonic == 'stp' and 'sp' in i.op_str and '!' in i.op_str:
        ann = " ; ← FUNC ENTRY"
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            names = {
                ROM_BASE + 0xD334: "zlp_send",
                ROM_BASE + 0xD368: "ep0_transfer_setup",
                ROM_BASE + 0xE0D8: "usb_core_do_transfer",
                ROM_BASE + 0xD2D0: "hal_abort_call",
            }
            if t in names:
                ann = f" ; → {names[t]}"
        except: pass
    if i.mnemonic == 'cmp' and '#' in i.op_str:
        try:
            val = int(i.op_str.split('#')[1].strip(), 0)
            ann = f" ; cmp with {val} (0x{val:x})"
        except: pass
    print(f"  0x{i.address:X}: {i.mnemonic:6s} {i.op_str}{ann}")
    if ret_count >= 2 and i.address > ROM_BASE + 0xDAD8:
        break

# ============================================================
# PART 3: Direct zlp_send caller at DDFC
# ============================================================
print("\n" + "=" * 70)
print("PART 3: Direct zlp_send caller at DDFC")
print("=" * 70)

# Find function prologue
for off in range(0xDDF0, 0xDD00, -4):
    instrs = disasm_at(off, 1)
    if instrs and instrs[0].mnemonic == 'stp' and 'sp' in instrs[0].op_str and '!' in instrs[0].op_str:
        func_start = off
        print(f"  Function prologue at 0x{ROM_BASE + off:X}")
        break
else:
    func_start = 0xDD90
    print(f"  Starting from 0x{ROM_BASE + func_start:X}")

ret_count = 0
for i in disasm_range(func_start, func_start + 0x200):
    ann = ""
    if i.mnemonic == 'ret':
        ann = " ; ← RET"
        ret_count += 1
    if i.mnemonic == 'stp' and 'sp' in i.op_str and '!' in i.op_str:
        ann = " ; ← FUNC ENTRY"
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            names = {
                ROM_BASE + 0xD334: "zlp_send",
                ROM_BASE + 0xD368: "ep0_transfer_setup",
                ROM_BASE + 0xE0D8: "usb_core_do_transfer",
                ROM_BASE + 0xD2D0: "hal_abort_call",
            }
            if t in names:
                ann = f" ; → {names[t]}"
        except: pass
    if i.mnemonic == 'cmp' and '#' in i.op_str:
        try:
            val = int(i.op_str.split('#')[1].strip(), 0)
            ann = f" ; cmp with {val} (0x{val:x})"
        except: pass
    print(f"  0x{i.address:X}: {i.mnemonic:6s} {i.op_str}{ann}")
    if ret_count >= 2 and i.address > ROM_BASE + 0xDDFC:
        break

# ============================================================
# PART 4: dwc3_callback_and_free (E0A8) — verify NULL callback behavior
# ============================================================
print("\n" + "=" * 70)
print("PART 4: dwc3_callback_and_free (E0A8) — NULL callback handling")
print("=" * 70)

for i in disasm_at(0xE0A8, 20):
    ann = ""
    if 'ldr' in i.mnemonic and '#0x20' in i.op_str:
        ann = " ; load callback from io_req+0x20"
    if i.mnemonic == 'cbz':
        ann = " ; ★ if callback==NULL, skip!"
    if i.mnemonic == 'blr':
        ann = " ; ★ CALL callback"
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            if t == ROM_BASE + 0xF468: ann = " ; → free"
        except: pass
    if i.mnemonic == 'ret':
        ann = " ; ← RET"
    print(f"  0x{i.address:X}: {i.mnemonic:6s} {i.op_str}{ann}")
    if i.mnemonic == 'ret':
        break

# ============================================================
# PART 5: DFU request dispatch — look at function calling E5DC
# The DFU handler that dispatches UPLOAD/DOWNLOAD/ABORT etc.
# ============================================================
print("\n" + "=" * 70)
print("PART 5: DFU request dispatch (D94C ep0_handler → DFU dispatch)")
print("=" * 70)

# The ep0_handler at D94C dispatches to different handlers based on request type
# Let's trace the DFU interface handler
# First, let's look at what's around E400-E600 (wider DFU area) with DFU bRequest checks
print("--- DFU dispatch area E400-E600 ---")
for i in disasm_range(0xE400, 0xE614):
    ann = ""
    if i.mnemonic == 'stp' and 'sp' in i.op_str and '!' in i.op_str:
        ann = " ; ← FUNC ENTRY"
    if i.mnemonic == 'ret':
        ann = " ; ← RET"
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            names = {
                ROM_BASE + 0xD334: "zlp_send",
                ROM_BASE + 0xD368: "ep0_transfer_setup",
                ROM_BASE + 0xE0D8: "usb_core_do_transfer",
                ROM_BASE + 0xE0A8: "dwc3_callback_and_free",
                ROM_BASE + 0xF3B0: "calloc",
                ROM_BASE + 0xF1EC: "malloc",
                ROM_BASE + 0xF468: "free",
                ROM_BASE + 0xF680: "memalign",
                ROM_BASE + 0x10D80: "bzero",
                ROM_BASE + 0x10E00: "memset",
                ROM_BASE + 0x10BD0: "memcpy",
            }
            if t in names:
                ann = f" ; → {names[t]}"
        except: pass
    if i.mnemonic == 'cmp' and '#' in i.op_str:
        try:
            val = int(i.op_str.split('#')[1].strip(), 0)
            dfu_reqs = {0: "DFU_DETACH", 1: "DFU_DNLOAD", 2: "DFU_UPLOAD", 
                       3: "DFU_GETSTATUS", 4: "DFU_CLRSTATUS", 5: "DFU_GETSTATE", 6: "DFU_ABORT"}
            if val in dfu_reqs:
                ann = f" ; ★ {dfu_reqs[val]}"
        except: pass
    print(f"  0x{i.address:X}: {i.mnemonic:6s} {i.op_str}{ann}")

# ============================================================
# PART 6: Wider EP0 handler (D94C) → look at standard request dispatch
# This shows what bmRequestType values trigger which paths
# ============================================================
print("\n" + "=" * 70)
print("PART 6: ep0_handler dispatch (D94C → D9D0+)")
print("=" * 70)

# The ep0_handler at D94C processes SETUP packets
# After the per-SETUP abort at D984, it dispatches based on bmRequestType
for i in disasm_range(0xD94C, 0xDA00):
    ann = ""
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            names = {
                ROM_BASE + 0xD334: "zlp_send",
                ROM_BASE + 0xD368: "ep0_transfer_setup",
                ROM_BASE + 0xD2D0: "hal_abort_call",
                ROM_BASE + 0xE158: "ep_abort_wrapper",
                ROM_BASE + 0xD3D0: "standard_device_request_cb",
                ROM_BASE + 0xE0D8: "usb_core_do_transfer",
            }
            if t in names:
                ann = f" ; → {names[t]}"
        except: pass
    if i.mnemonic == 'cmp' and '#' in i.op_str:
        try:
            val = int(i.op_str.split('#')[1].strip(), 0)
            # bmRequestType: 0x00=std OUT, 0x80=std IN, 0x01=class OUT, 0x21=class IF OUT, 0xA1=class IF IN
            req_types = {
                0x00: "STD_OUT_DEV", 0x80: "STD_IN_DEV", 0x01: "STD_OUT_IF",
                0x81: "STD_IN_IF", 0x02: "STD_OUT_EP", 0x82: "STD_IN_EP",
                0x21: "CLASS_IF_OUT", 0xA1: "CLASS_IF_IN",
                0x40: "VENDOR_OUT", 0xC0: "VENDOR_IN",
            }
            if val in req_types:
                ann = f" ; ★ bmRequestType = {req_types[val]}"
            elif val < 0x10:
                ann = f" ; check = {val}"
        except: pass
    if 'tbz' in i.mnemonic or 'tbnz' in i.mnemonic:
        ann = " ; bit test"
    if i.mnemonic == 'stp' and 'sp' in i.op_str and '!' in i.op_str:
        ann = " ; ← FUNC ENTRY"
    print(f"  0x{i.address:X}: {i.mnemonic:6s} {i.op_str}{ann}")

# Also the standard request handler from D9D0 to ~DB00
print("\n--- Standard request dispatch (DA00-DB80) ---")
for i in disasm_range(0xDA00, 0xDB80):
    ann = ""
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            names = {
                ROM_BASE + 0xD334: "zlp_send",
                ROM_BASE + 0xD368: "ep0_transfer_setup",
                ROM_BASE + 0xD2D0: "hal_abort_call",
                ROM_BASE + 0xE0D8: "usb_core_do_transfer",
            }
            if t in names:
                ann = f" ; → {names[t]}"
        except: pass
    if i.mnemonic == 'cmp' and '#' in i.op_str:
        try:
            val = int(i.op_str.split('#')[1].strip(), 0)
            std_reqs = {
                0: "GET_STATUS", 1: "CLEAR_FEATURE", 3: "SET_FEATURE",
                5: "SET_ADDRESS", 6: "GET_DESCRIPTOR", 7: "SET_DESCRIPTOR",
                8: "GET_CONFIGURATION", 9: "SET_CONFIGURATION"
            }
            if val in std_reqs:
                ann = f" ; ★ bRequest = {std_reqs[val]}"
        except: pass
    if 'ldr' in i.mnemonic and '#0x28' in i.op_str:
        ann += " ; [+0x28]"
    if i.mnemonic == 'ret':
        ann = " ; ← RET"
    if i.mnemonic == 'stp' and 'sp' in i.op_str and '!' in i.op_str:
        ann = " ; ← FUNC ENTRY"
    print(f"  0x{i.address:X}: {i.mnemonic:6s} {i.op_str}{ann}")

# ============================================================
# PART 7: Full context for the DD88 caller
# Trace the larger function containing DD88
# ============================================================
print("\n" + "=" * 70)
print("PART 7: Broader DD function (DD00-DE00) — standard request IN handler?")
print("=" * 70)

for i in disasm_range(0xDC00, 0xDE10):
    ann = ""
    if i.mnemonic == 'stp' and 'sp' in i.op_str and '!' in i.op_str:
        ann = " ; ← FUNC ENTRY"
    if i.mnemonic == 'ret':
        ann = " ; ← RET"
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            names = {
                ROM_BASE + 0xD334: "zlp_send",
                ROM_BASE + 0xD368: "ep0_transfer_setup",
                ROM_BASE + 0xE0D8: "usb_core_do_transfer",
                ROM_BASE + 0xD2D0: "hal_abort_call",
                ROM_BASE + 0x8978: "panic",
                ROM_BASE + 0xF1EC: "malloc",
                ROM_BASE + 0xF680: "memalign",
            }
            if t in names:
                ann = f" ; → {names[t]}"
        except: pass
    if i.mnemonic == 'adr' and '#' in i.op_str:
        try:
            target = int(i.op_str.split('#')[1].strip(), 0)
            if target == ROM_BASE + 0xD3D0:
                ann = " ; ★★★ D3D0 = standard_device_request_cb ★★★"
        except: pass
    if i.mnemonic == 'cmp' and '#' in i.op_str:
        try:
            val = int(i.op_str.split('#')[1].strip(), 0)
            desc_types = {1: "DEVICE", 2: "CONFIGURATION", 3: "STRING",
                         6: "DEVICE_QUALIFIER", 15: "BOS"}
            if val in desc_types:
                ann = f" ; ★ descriptor type = {desc_types[val]}"
            elif val <= 20:
                ann = f" ; check = {val}"
        except: pass
    # mov with specific sizes
    if i.mnemonic == 'mov' and '#0x' in i.op_str:
        try:
            val = int(i.op_str.split('#')[1].strip(), 0)
            if val in (18, 27, 0x200, 0x800):
                ann = f" ; size = {val} (0x{val:x})"
        except: pass
    if 'ldr' in i.mnemonic and '#0x28' in i.op_str:
        ann += " ; [+0x28]"
    print(f"  0x{i.address:X}: {i.mnemonic:6s} {i.op_str}{ann}")

print("\n\nDone.")
